mod config;
mod pages;
mod routes;

use config::Config;

#[tokio::main]
async fn main() {
    let config = Config::from_env();

    let listener = tokio::net::TcpListener::bind(config.bind_addr)
        .await
        .unwrap_or_else(|error| panic!("failed to bind {}: {error}", config.bind_addr));

    println!("listening on http://{}", config.bind_addr);

    axum::serve(listener, routes::router())
        .await
        .expect("server failed");
}
