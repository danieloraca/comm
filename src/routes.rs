use axum::{
    Router,
    routing::{get, post},
};

use crate::{auth, pages};

pub fn router(state: auth::AppState) -> Router {
    Router::new()
        .route("/", get(pages::login_page))
        .route("/login", post(auth::login))
        .route("/chat", get(auth::chat))
        .with_state(state)
}
