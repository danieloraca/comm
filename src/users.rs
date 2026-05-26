use std::{env, fs, sync::Arc};

use argon2::{Argon2, PasswordHash, PasswordVerifier};
use serde::Deserialize;

const DEFAULT_USERS_FILE: &str = "users.toml";

#[derive(Clone, Default)]
pub struct UserStore {
    users: Arc<Vec<User>>,
}

impl UserStore {
    pub fn load_from_env() -> Self {
        let path = env::var("COMM_USERS_FILE").unwrap_or_else(|_| DEFAULT_USERS_FILE.to_string());
        let contents = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("failed to read users file `{path}`: {error}"));
        let file: UsersFile = toml::from_str(&contents)
            .unwrap_or_else(|error| panic!("failed to parse users file `{path}`: {error}"));

        if file.user.len() != 2 {
            panic!("users file `{path}` must contain exactly two [[user]] entries");
        }

        Self {
            users: Arc::new(
                file.user
                    .into_iter()
                    .map(|entry| User {
                        username: entry.username,
                        password_hash: entry.password_hash,
                    })
                    .collect(),
            ),
        }
    }

    pub fn verify_credentials(&self, username: &str, password: &str) -> bool {
        let Some(user) = self.users.iter().find(|user| user.username == username) else {
            return false;
        };

        let Ok(hash) = PasswordHash::new(&user.password_hash) else {
            return false;
        };

        Argon2::default()
            .verify_password(password.as_bytes(), &hash)
            .is_ok()
    }
}

struct User {
    username: String,
    password_hash: String,
}

#[derive(Deserialize)]
struct UsersFile {
    user: Vec<UserEntry>,
}

#[derive(Deserialize)]
struct UserEntry {
    username: String,
    password_hash: String,
}
