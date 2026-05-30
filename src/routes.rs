use axum::{
    Router,
    routing::{get, post},
};

use crate::{auth, chat, pages};

pub fn router(state: auth::AppState) -> Router {
    Router::new()
        .route("/", get(pages::login_page))
        .route("/login", post(auth::login))
        .route("/logout", post(auth::logout))
        .route("/verify-password", post(auth::verify_password))
        .route("/chat", get(auth::chat))
        .route("/ws", get(chat::websocket))
        .with_state(state)
}
