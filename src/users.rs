struct User {
    username: &'static str,
    password: &'static str,
}

const USERS: &[User] = &[
    User {
        username: "u1",
        password: "u1p",
    },
    User {
        username: "u2",
        password: "u2p",
    },
];

pub fn verify_credentials(username: &str, password: &str) -> bool {
    USERS
        .iter()
        .any(|user| user.username == username && user.password == password)
}
