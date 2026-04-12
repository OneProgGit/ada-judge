use ada_judge_public_models::verdicts::TotalVerdict;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use models::users::JwtClaims;
use tools::map::MapLogExt;

pub fn create_jwt(claims: &JwtClaims, secret: &str) -> Result<String, TotalVerdict> {
    encode(
        &Header::default(),
        claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_log(TotalVerdict::Bug)
}

pub fn decode_jwt(token: &str, secret: &str) -> Result<JwtClaims, TotalVerdict> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;

    let claims = decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_log(TotalVerdict::InvalidRequest)?;

    Ok(claims.claims)
}
