use argon2::{
    Argon2, PasswordHasher,
    password_hash::{SaltString, rand_core::OsRng},
};
use axum::http::StatusCode;
use models::verdicts::TotalVerdict;
use tools::map::{MapHttpExt, MapLogExt};

pub fn get_password_hash(password: &str) -> Result<String, StatusCode> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let res = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_log(TotalVerdict::Bug)
        .map_http()?;

    Ok(res.to_string())
}
