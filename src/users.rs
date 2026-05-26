struct User {
    username: &'static str,
    password: &'static str,
}

const USERS: &[User] = &[
    User {
        username: "daniel",
        password: "change-me-daniel",
    },
    User {
        username: "friend",
        password: "change-me-friend",
    },
];

pub fn verify_credentials(username: &str, password: &str) -> bool {
    USERS
        .iter()
        .any(|user| user.username == username && user.password == password)
}
