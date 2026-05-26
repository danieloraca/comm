use argon2::{Argon2, PasswordHasher};
use password_hash::SaltString;
use rand_core::OsRng;

fn main() {
    let password = std::env::args().nth(1).unwrap_or_else(|| {
        rpassword::prompt_password("Password: ").expect("failed to read password")
    });
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .expect("failed to hash password");

    println!("{hash}");
}
