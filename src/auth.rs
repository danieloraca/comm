use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
    time::{Duration, Instant},
};

use axum::{
    Form,
    extract::State,
    http::{
        HeaderMap, HeaderValue,
        header::{COOKIE, SET_COOKIE},
    },
    response::{IntoResponse, Redirect, Response},
};
use rand::random;
use serde::Deserialize;
use tokio::sync::broadcast;

use crate::{chat, pages, store, users};

const MAX_FAILED_LOGIN_ATTEMPTS: u32 = 5;
const LOGIN_COOLDOWN: Duration = Duration::from_secs(60);
const SESSION_COOKIE: &str = "comm_session";

#[derive(Clone)]
pub struct AppState {
    chat_tx: broadcast::Sender<chat::ChatMessage>,
    login_attempts: Arc<RwLock<HashMap<String, LoginAttempt>>>,
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

pub async fn login(State(state): State<AppState>, Form(form): Form<LoginForm>) -> Response {
    if login_is_rate_limited(&state, &form.username) {
        return Redirect::to("/?error=rate_limited").into_response();
    }

    if !state
        .users
        .verify_credentials(&form.username, &form.password)
    {
        record_failed_login(&state, &form.username);
        return Redirect::to("/?error=1").into_response();
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
    match state.authenticated_user(&headers) {
        Some(username) => pages::chat_page(&username).into_response(),
        None => Redirect::to("/").into_response(),
    }
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
    response
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

    pub fn chat_sender(&self) -> broadcast::Sender<chat::ChatMessage> {
        self.chat_tx.clone()
    }

    pub fn message_store(&self) -> store::MessageStore {
        self.store.clone()
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
