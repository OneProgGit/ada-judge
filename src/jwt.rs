use aj_models::errors::AdaJudgeError;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use models::users::JwtClaims;

pub fn create_jwt(claims: &JwtClaims, secret: &str) -> Result<String, AdaJudgeError> {
    encode(
        &Header::default(),
        claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| AdaJudgeError::Internal)
}

pub fn decode_jwt(token: &str, secret: &str) -> Result<JwtClaims, AdaJudgeError> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;

    let claims = decode::<JwtClaims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|_| AdaJudgeError::InvalidJwt)?;

    Ok(claims.claims)
}
