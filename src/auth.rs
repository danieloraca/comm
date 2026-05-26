use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
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

use crate::{pages, users};

const SESSION_COOKIE: &str = "comm_session";

#[derive(Clone, Default)]
pub struct AppState {
    sessions: Arc<RwLock<HashMap<String, String>>>,
    users: users::UserStore,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            sessions: Arc::default(),
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
    if !state
        .users
        .verify_credentials(&form.username, &form.password)
    {
        return Redirect::to("/?error=1").into_response();
    }

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
    match authenticated_user(&state, &headers) {
        Some(username) => pages::chat_page(&username).into_response(),
        None => Redirect::to("/").into_response(),
    }
}

fn authenticated_user(state: &AppState, headers: &HeaderMap) -> Option<String> {
    let token = session_token(headers)?;
    state
        .sessions
        .read()
        .expect("session store lock poisoned")
        .get(&token)
        .cloned()
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

fn create_session_token() -> String {
    random::<[u8; 32]>()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
