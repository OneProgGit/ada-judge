use crate::crypt::get_password_hash;
use crate::jwt::create_jwt;
use crate::{app_state::AppState, crypt::verify_password};
use axum::{Json, extract::State, http::StatusCode};
use chrono::{Duration, Utc};
use models::users::JwtClaims;
use models::{
    users::{LoginRequest, RegisterRequest},
    verdicts::TotalVerdict,
};
use std::{env, sync::Arc};
use tools::map::{MapHttpExt, MapLogExt};

pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(user): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<i64>), StatusCode> {
    log::info!("Get master password");

    let master_password = env::var("MASTER_PASSWORD")
        .map_log(TotalVerdict::Bug)
        .map_http()?;

    log::info!("Check master password");
    if user.master_password.trim() != master_password.trim() {
        log::error!("Incorrect master password");
        return Err(StatusCode::BAD_REQUEST);
    }

    log::info!("Get password hash");
    let password_hash = get_password_hash(&user.password).map_http()?;

    log::info!("Create user");
    let user_id = database::auth::create_user(&state.db, &user.login, &password_hash)
        .await
        .map_http()?;

    Ok((StatusCode::OK, Json(user_id)))
}

#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
pub async fn login(
    State(state): State<Arc<AppState>>,
    Json(user): Json<LoginRequest>,
) -> Result<(StatusCode, Json<String>), StatusCode> {
    log::info!("Get expected user");
    let expected_user = database::auth::get_user_by_login(&state.db, &user.login)
        .await
        .map_http()?;

    log::info!("Verify password");
    let is_valid_password =
        verify_password(&expected_user.password_hash, &user.password).map_http()?;

    if !is_valid_password {
        log::error!("Invalid password");
        return Err(StatusCode::BAD_REQUEST);
    }

    log::info!("Get jwt exp hours");
    let jwt_exp_hours = env::var("JWT_EXP_HOURS");

    let jwt_exp_hours = match jwt_exp_hours {
        Ok(s) => s.parse().map_log(TotalVerdict::Bug).map_http()?,
        Err(_) => 24,
    };

    log::info!("Create jwt claims");
    let claims = JwtClaims {
        id: expected_user.id,
        exp: (Utc::now() + Duration::hours(jwt_exp_hours)).timestamp() as usize,
    };
    let secret = env::var("JWT_SECRET")
        .map_log(TotalVerdict::Bug)
        .map_http()?;

    Ok((
        StatusCode::OK,
        Json(create_jwt(&claims, &secret).map_http()?),
    ))
}
