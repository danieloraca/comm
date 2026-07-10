mod attachment_crypto;
mod auth;
mod chat;
mod clock;
mod config;
mod crypto;
mod link_preview;
mod notifier;
mod pages;
mod routes;
mod store;
mod users;

use config::Config;

#[tokio::main]
async fn main() {
    let config = Config::from_env();

    let listener = tokio::net::TcpListener::bind(config.bind_addr)
        .await
        .unwrap_or_else(|error| panic!("failed to bind {}: {error}", config.bind_addr));

    println!("listening on http://{}", config.bind_addr);
    println!("timestamps use timezone {}", clock::configured_timezone());

    let state = auth::AppState::new().await;

    axum::serve(listener, routes::router(state))
        .await
        .expect("server failed");
}
