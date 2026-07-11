use crate::crypt::get_password_hash;
use crate::jwt::create_jwt;
use crate::middleware::auth::Auth;
use crate::{app_state::AppState, crypt::verify_password};
use ada_judge_public_models::DeletionRequest;
use ada_judge_public_models::users::{LoginRequest, RegisterRequest};
use ada_judge_public_models::verdicts::TotalVerdict;
use axum::{Json, extract::State, http::StatusCode};
use chrono::{Duration, Utc};
use models::users::JwtClaims;
use std::env;
use tools::map::{MapHttpExt, MapLogExt};

pub async fn register(
    State(state): State<AppState>,
    Json(user): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<i64>), StatusCode> {
    log::info!("Get password hash");
    let password_hash = get_password_hash(&user.password).map_http()?;

    log::info!("Create user");
    let user_id = database::users::create_user(&state.db, &user.login, &password_hash)
        .await
        .map_http()?;

    Ok((StatusCode::OK, Json(user_id)))
}

#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
pub async fn login(
    State(state): State<AppState>,
    Json(user): Json<LoginRequest>,
) -> Result<(StatusCode, Json<String>), StatusCode> {
    log::info!("Get expected user");
    let expected_user = database::users::get_user_by_login(&state.db, &user.login)
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

pub async fn delete_my_account(
    State(state): State<AppState>,
    Auth(auth): Auth,
    Json(request): Json<DeletionRequest>,
) -> Result<(), StatusCode> {
    if request.login != auth.login
        || request.password != request.password_confirmation
        || !request.deletion_confirmation
    {
        return Err(StatusCode::BAD_REQUEST);
    }

    log::info!("Verify password");
    let is_valid_password = verify_password(&auth.password_hash, &request.password).map_http()?;

    if !is_valid_password {
        log::error!("Invalid password");
        Err(StatusCode::BAD_REQUEST)
    } else {
        database::users::delete_user(&state.db, auth.id)
            .await
            .map_http()?;
        Ok(())
    }
}
