use crate::api::ApiError;
use crate::crypt::get_password_hash;
use crate::jwt::create_jwt;
use crate::middleware::auth::Auth;
use crate::{app_state::AppState, crypt::verify_password};
use aj_models::DeletionRequest;
use aj_models::errors::{AdaJudgeError, AuthError, Deletion};
use aj_models::users::{AdminLevel, LoginRequest, RegisterRequest};
use axum::{Json, extract::State};
use chrono::{Duration, Utc};
use models::users::JwtClaims;
use std::env;
use tools::map::MapHttpExt;

pub async fn register(
    State(state): State<AppState>,
    Json(user): Json<RegisterRequest>,
) -> Result<(), ApiError> {
    if user.password != user.password_confirmation {
        return Err(AdaJudgeError::Auth(AuthError::PasswordsDontMatch)).map_http()?;
    }

    let password_hash = get_password_hash(&user.password).map_http()?;
    let user_id = database::users::create_user(&state.db, &user.login, &password_hash)
        .await
        .map_http()?;
    if user_id == 1 {
        database::users::change_admin_level(&state.db, user_id, &AdminLevel::Owner)
            .await
            .map_http()?;
    }

    Ok(())
}

#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
pub async fn login(
    State(state): State<AppState>,
    Json(user): Json<LoginRequest>,
) -> Result<Json<String>, ApiError> {
    let expected_user = database::users::get_user_by_login(&state.db, &user.login)
        .await
        .map_http()?;
    let is_valid_password =
        verify_password(&expected_user.password_hash, &user.password).map_http()?;

    if !is_valid_password {
        return Err(AdaJudgeError::Auth(AuthError::InvalidLoginOrPassword)).map_http()?;
    }
    let jwt_exp_hours = env::var("JWT_EXP_HOURS");

    let jwt_exp_hours = match jwt_exp_hours {
        Ok(s) => s.parse().map_err(|_| AdaJudgeError::Internal).map_http()?,
        Err(_) => 24,
    };
    let claims = JwtClaims {
        id: expected_user.id,
        exp: (Utc::now() + Duration::hours(jwt_exp_hours)).timestamp() as usize,
    };
    let secret = env::var("JWT_SECRET")
        .map_err(|_| AdaJudgeError::Internal)
        .map_http()?;

    Ok(Json(create_jwt(&claims, &secret).map_http()?))
}

pub async fn delete_my_account(
    State(state): State<AppState>,
    Auth(auth): Auth,
    Json(request): Json<DeletionRequest>,
) -> Result<(), ApiError> {
    if request.login != auth.login {
        return Err(AdaJudgeError::Deletion(Deletion::InvalidLoginOrPassword)).map_http()?;
    }
    if !request.deletion_confirmation {
        return Err(AdaJudgeError::Deletion(
            Deletion::MissingDeletionConfirmation,
        ))
        .map_http()?;
    }
    let is_valid_password = verify_password(&auth.password_hash, &request.password).map_http()?;

    if is_valid_password {
        database::users::delete_user(&state.db, auth.id)
            .await
            .map_http()?;
        Ok(())
    } else {
        Err(AdaJudgeError::Deletion(Deletion::InvalidLoginOrPassword)).map_http()?
    }
}
