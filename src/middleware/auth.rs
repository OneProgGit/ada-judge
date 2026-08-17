use crate::{app_state::AppState, jwt::decode_jwt};
use aj_models::errors::AdaJudgeError;
use axum::{
    Json,
    extract::{FromRef, FromRequestParts},
    http::{StatusCode, request::Parts},
};
use models::users::DatabaseUser;
use std::env;
use tools::map::MapHttpExt;

pub struct Auth(pub DatabaseUser);

impl<S> FromRequestParts<S> for Auth
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = (StatusCode, Json<AdaJudgeError>);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let state = AppState::from_ref(state);

        let token = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .ok_or(Err(AdaJudgeError::InvalidJwt).map_http()?)?;

        let secret = env::var("JWT_SECRET")
            .map_err(|_| AdaJudgeError::Internal)
            .map_http()?;
        let claims = decode_jwt(token, &secret).map_http()?;

        let user = database::users::get_user_by_id(&state.db, claims.id)
            .await
            .map_http()?;

        Ok(Self(user))
    }
}
