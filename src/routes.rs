use axum::{Router, routing::get};

use crate::pages;

pub fn router() -> Router {
    Router::new().route("/", get(pages::login_page))
}
