use aj_models::verdicts::TestingVerdict;
use argon2::{
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
    password_hash::{SaltString, rand_core::OsRng},
};
use tools::map::MapLogExt;

pub fn get_password_hash(password: &str) -> Result<String, TestingVerdict> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let res = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_log(TestingVerdict::Bug)?;

    Ok(res.to_string())
}

pub fn verify_password(expected: &str, password: &str) -> Result<bool, TestingVerdict> {
    let hash = PasswordHash::new(expected).map_log(TestingVerdict::Bug)?;
    let argon2 = Argon2::default();

    Ok(argon2.verify_password(password.as_bytes(), &hash).is_ok())
}
