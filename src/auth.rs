use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use axum::{
    Form, Json,
    extract::{Multipart, Path, State},
    http::{
        HeaderMap, HeaderValue, StatusCode,
        header::{CACHE_CONTROL, CONTENT_TYPE, COOKIE, PRAGMA, SET_COOKIE},
    },
    response::{IntoResponse, Redirect, Response},
};
use rand::random;
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

use crate::{chat, pages, store, users};

const MAX_FAILED_LOGIN_ATTEMPTS: u32 = 5;
const MAX_ATTACHMENT_BYTES: usize = 20 * 1024 * 1024;
const LOGIN_COOLDOWN: Duration = Duration::from_secs(60);
const SESSION_COOKIE: &str = "comm_session";

#[derive(Clone)]
pub struct AppState {
    chat_tx: broadcast::Sender<chat::ServerEvent>,
    login_attempts: Arc<RwLock<HashMap<String, LoginAttempt>>>,
    presence_counts: Arc<RwLock<HashMap<String, usize>>>,
    sessions: Arc<RwLock<HashMap<String, String>>>,
    store: store::MessageStore,
    users: users::UserStore,
}

impl AppState {
    pub async fn new() -> Self {
        let (chat_tx, _) = broadcast::channel(100);

        Self {
            chat_tx,
            login_attempts: Arc::default(),
            presence_counts: Arc::default(),
            sessions: Arc::default(),
            store: store::MessageStore::load_from_env().await,
            users: users::UserStore::load_from_env(),
        }
    }
}

#[derive(Deserialize)]
pub struct LoginForm {
    username: String,
    password: String,
}

#[derive(Deserialize)]
pub struct LoginStartForm {
    q: String,
}

#[derive(Deserialize)]
pub struct VerifyPasswordForm {
    password: String,
}

#[derive(Serialize)]
pub struct AttachmentUploadResponse {
    id: i64,
    mime_type: String,
    original_name: Option<String>,
    size_bytes: i64,
}

pub async fn start_login(
    State(state): State<AppState>,
    Form(form): Form<LoginStartForm>,
) -> Response {
    Redirect::to(&login_start_redirect_location(&state.users, &form.q)).into_response()
}

pub async fn login(State(state): State<AppState>, Form(form): Form<LoginForm>) -> Response {
    if login_is_rate_limited(&state, &form.username) {
        return Redirect::to(&login_error_redirect_location(
            &form.username,
            "rate_limited",
        ))
        .into_response();
    }

    if !state
        .users
        .verify_credentials(&form.username, &form.password)
    {
        record_failed_login(&state, &form.username);
        return Redirect::to(&login_error_redirect_location(&form.username, "1")).into_response();
    }

    clear_failed_logins(&state, &form.username);

    let token = create_session_token();
    state
        .sessions
        .write()
        .expect("session store lock poisoned")
        .insert(token.clone(), form.username);

    let mut headers = HeaderMap::new();
    headers.insert(SET_COOKIE, session_cookie(&token));

    (headers, Redirect::to("/chat")).into_response()
}

pub async fn chat(State(state): State<AppState>, headers: HeaderMap) -> Response {
    no_store_response(match state.authenticated_user(&headers) {
        Some(username) => pages::chat_page(&username).into_response(),
        None => Redirect::to("/").into_response(),
    })
}

pub async fn logout(State(state): State<AppState>, headers: HeaderMap) -> Response {
    if let Some(token) = session_token(&headers) {
        state
            .sessions
            .write()
            .expect("session store lock poisoned")
            .remove(&token);
    }

    let mut response = Redirect::to("/").into_response();
    response
        .headers_mut()
        .insert(SET_COOKIE, expired_session_cookie());
    no_store_response(response)
}

pub async fn upload_attachment(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut multipart: Multipart,
) -> Response {
    let Some(username) = state.authenticated_user(&headers) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    while let Ok(Some(field)) = multipart.next_field().await {
        if field.name() != Some("photo") {
            continue;
        }

        let mime_type = field.content_type().map(str::to_owned).unwrap_or_default();
        if !allowed_image_mime_type(&mime_type) {
            return StatusCode::UNSUPPORTED_MEDIA_TYPE.into_response();
        }

        let original_name = field.file_name().map(str::to_owned);
        let Ok(bytes) = field.bytes().await else {
            return StatusCode::BAD_REQUEST.into_response();
        };

        if bytes.is_empty() || bytes.len() > MAX_ATTACHMENT_BYTES {
            return StatusCode::PAYLOAD_TOO_LARGE.into_response();
        }

        let Ok(attachment) = state
            .store
            .save_attachment(&username, original_name.as_deref(), &mime_type, &bytes)
            .await
        else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };

        return Json(AttachmentUploadResponse {
            id: attachment.id,
            mime_type: attachment.mime_type,
            original_name: attachment.original_name,
            size_bytes: attachment.size_bytes,
        })
        .into_response();
    }

    StatusCode::BAD_REQUEST.into_response()
}

pub async fn attachment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<i64>,
) -> Response {
    let Some(username) = state.authenticated_user(&headers) else {
        return StatusCode::UNAUTHORIZED.into_response();
    };

    let Ok(Some(attachment)) = state.store.attachment_for_user(&username, id).await else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let Ok(content_type) = HeaderValue::from_str(&attachment.mime_type) else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    ([(CONTENT_TYPE, content_type)], attachment.bytes).into_response()
}

pub async fn verify_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    Form(form): Form<VerifyPasswordForm>,
) -> StatusCode {
    let Some(username) = state.authenticated_user(&headers) else {
        return StatusCode::UNAUTHORIZED;
    };

    if login_is_rate_limited(&state, &username) {
        return StatusCode::TOO_MANY_REQUESTS;
    }

    if !state.users.verify_credentials(&username, &form.password) {
        record_failed_login(&state, &username);
        return StatusCode::UNAUTHORIZED;
    }

    clear_failed_logins(&state, &username);
    StatusCode::NO_CONTENT
}

fn allowed_image_mime_type(mime_type: &str) -> bool {
    matches!(
        mime_type,
        "image/jpeg" | "image/png" | "image/webp" | "image/gif" | "image/heic" | "image/heif"
    )
}

impl AppState {
    pub fn authenticated_user(&self, headers: &HeaderMap) -> Option<String> {
        let token = session_token(headers)?;
        self.sessions
            .read()
            .expect("session store lock poisoned")
            .get(&token)
            .cloned()
    }

    pub fn chat_sender(&self) -> broadcast::Sender<chat::ServerEvent> {
        self.chat_tx.clone()
    }

    pub fn message_store(&self) -> store::MessageStore {
        self.store.clone()
    }

    pub fn connect_user(&self, username: &str) -> bool {
        let mut counts = self
            .presence_counts
            .write()
            .expect("presence store lock poisoned");
        let count = counts.entry(username.to_owned()).or_insert(0);
        let was_offline = *count == 0;
        *count += 1;
        was_offline
    }

    pub fn disconnect_user(&self, username: &str) -> bool {
        let mut counts = self
            .presence_counts
            .write()
            .expect("presence store lock poisoned");
        let Some(count) = counts.get_mut(username) else {
            return false;
        };

        *count = count.saturating_sub(1);

        if *count == 0 {
            counts.remove(username);
            return true;
        }

        false
    }

    pub fn online_users(&self) -> Vec<String> {
        self.presence_counts
            .read()
            .expect("presence store lock poisoned")
            .keys()
            .cloned()
            .collect()
    }
}

fn session_token(headers: &HeaderMap) -> Option<String> {
    let cookie_header = headers.get(COOKIE)?.to_str().ok()?;
    cookie_header
        .split(';')
        .map(str::trim)
        .find_map(|cookie| cookie.strip_prefix("comm_session=").map(str::to_owned))
}

fn session_cookie(token: &str) -> HeaderValue {
    let cookie =
        format!("{SESSION_COOKIE}={token}; HttpOnly; SameSite=Strict; Path=/; Max-Age=86400");
    HeaderValue::from_str(&cookie).expect("session cookie should be a valid header value")
}

fn expired_session_cookie() -> HeaderValue {
    HeaderValue::from_static(
        "comm_session=; HttpOnly; SameSite=Strict; Path=/; Max-Age=0; Expires=Thu, 01 Jan 1970 00:00:00 GMT",
    )
}

fn no_store_response(mut response: Response) -> Response {
    response.headers_mut().insert(
        CACHE_CONTROL,
        HeaderValue::from_static("no-store, no-cache, must-revalidate, max-age=0"),
    );
    response
        .headers_mut()
        .insert(PRAGMA, HeaderValue::from_static("no-cache"));
    response
}

fn create_session_token() -> String {
    random::<[u8; 32]>()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[derive(Clone, Copy)]
struct LoginAttempt {
    failed_count: u32,
    locked_until: Option<Instant>,
}

fn login_is_rate_limited(state: &AppState, username: &str) -> bool {
    let mut attempts = state
        .login_attempts
        .write()
        .expect("login attempt store lock poisoned");
    let Some(attempt) = attempts.get(username).copied() else {
        return false;
    };

    match attempt.locked_until {
        Some(locked_until) if Instant::now() < locked_until => true,
        Some(_) => {
            attempts.remove(username);
            false
        }
        None => false,
    }
}

fn record_failed_login(state: &AppState, username: &str) {
    let mut attempts = state
        .login_attempts
        .write()
        .expect("login attempt store lock poisoned");
    let attempt = attempts.entry(username.to_owned()).or_insert(LoginAttempt {
        failed_count: 0,
        locked_until: None,
    });

    attempt.failed_count += 1;

    if attempt.failed_count >= MAX_FAILED_LOGIN_ATTEMPTS {
        attempt.locked_until = Some(Instant::now() + LOGIN_COOLDOWN);
    }
}

fn clear_failed_logins(state: &AppState, username: &str) {
    state
        .login_attempts
        .write()
        .expect("login attempt store lock poisoned")
        .remove(username);
}

fn login_start_redirect_location(users: &users::UserStore, query: &str) -> String {
    let query = query.trim();

    if query.is_empty() {
        return "/".to_string();
    }

    if users.has_username(query) {
        return format!("/?username={}", percent_encode(query));
    }

    format!("https://www.google.com/search?q={}", percent_encode(query))
}

fn login_error_redirect_location(username: &str, error: &str) -> String {
    format!(
        "/?username={}&error={}",
        percent_encode(username),
        percent_encode(error)
    )
}

fn percent_encode(value: &str) -> String {
    value
        .bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                vec![byte as char]
            }
            _ => format!("%{byte:02X}").chars().collect(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn start_login_known_username_goes_to_password_step() {
        let users = users::UserStore::from_usernames_for_tests(&["alice", "bob"]);

        assert_eq!(
            login_start_redirect_location(&users, "alice"),
            "/?username=alice"
        );
    }

    #[test]
    fn start_login_unknown_username_goes_to_google_search() {
        let users = users::UserStore::from_usernames_for_tests(&["alice", "bob"]);

        assert_eq!(
            login_start_redirect_location(&users, "weather london"),
            "https://www.google.com/search?q=weather%20london"
        );
    }

    #[test]
    fn start_login_trims_input_before_routing() {
        let users = users::UserStore::from_usernames_for_tests(&["alice", "bob"]);

        assert_eq!(
            login_start_redirect_location(&users, "  bob  "),
            "/?username=bob"
        );
    }

    #[test]
    fn start_login_empty_input_returns_to_login_page() {
        let users = users::UserStore::from_usernames_for_tests(&["alice", "bob"]);

        assert_eq!(login_start_redirect_location(&users, "   "), "/");
    }

    #[test]
    fn failed_password_returns_to_password_step() {
        assert_eq!(
            login_error_redirect_location("alice", "1"),
            "/?username=alice&error=1"
        );
    }

    #[test]
    fn rate_limited_login_returns_to_password_step() {
        assert_eq!(
            login_error_redirect_location("alice", "rate_limited"),
            "/?username=alice&error=rate_limited"
        );
    }

    #[test]
    fn redirect_values_are_percent_encoded() {
        assert_eq!(percent_encode("a b@example"), "a%20b%40example");
        assert_eq!(
            login_error_redirect_location("a b@example", "rate limited"),
            "/?username=a%20b%40example&error=rate%20limited"
        );
    }
}
