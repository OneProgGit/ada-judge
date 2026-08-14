use crate::{app_state::AppState, jwt::decode_jwt};
use aj_models::verdicts::TestingVerdict;
use axum::{
    extract::{FromRef, FromRequestParts},
    http::{StatusCode, request::Parts},
};
use models::users::DatabaseUser;
use std::env;
use tools::map::{MapHttpExt, MapLogExt};

pub struct Auth(pub DatabaseUser);

impl<S> FromRequestParts<S> for Auth
where
    AppState: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = StatusCode;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let state = AppState::from_ref(state);

        let token = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.strip_prefix("Bearer "))
            .ok_or(StatusCode::UNAUTHORIZED)?;

        let secret = env::var("JWT_SECRET")
            .map_log(TestingVerdict::Bug)
            .map_http()?;
        let claims = decode_jwt(token, &secret).map_http()?;

        let user = database::users::get_user_by_id(&state.db, claims.id)
            .await
            .map_http()?;

        Ok(Self(user))
    }
}
