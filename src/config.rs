use std::{env, net::SocketAddr};

const DEFAULT_BIND_ADDR: &str = "127.0.0.1:8787";

pub struct Config {
    pub bind_addr: SocketAddr,
}

impl Config {
    pub fn from_env() -> Self {
        let bind_addr =
            env::var("COMM_BIND_ADDR").unwrap_or_else(|_| DEFAULT_BIND_ADDR.to_string());
        let bind_addr = bind_addr
            .parse()
            .unwrap_or_else(|error| panic!("invalid COMM_BIND_ADDR `{bind_addr}`: {error}"));

        Self { bind_addr }
    }
}
