use crate::{api::ApiError, app_state::AppState, crypt::verify_password, middleware::auth::Auth};
use aj_models::{
    DeletionRequest,
    errors::{AdaJudgeError, Deletion},
    users::{AdminLevel, PrivateUserData, PublicUserData},
};
use axum::{
    Json,
    extract::{Path, State},
};
use tools::map::MapHttpExt;

pub async fn get_public_user_profile(
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
) -> Result<Json<PublicUserData>, ApiError> {
    Ok(Json(
        database::users::get_user_by_id(&state.db, user_id)
            .await
            .map_http()?
            .into(),
    ))
}

pub async fn get_my_user_profile(
    State(state): State<AppState>,
    Auth(auth): Auth,
) -> Result<Json<PrivateUserData>, ApiError> {
    Ok(Json(
        database::users::get_user_by_id(&state.db, auth.id)
            .await
            .map_http()?
            .into(),
    ))
}

pub async fn get_users(
    State(state): State<AppState>,
) -> Result<Json<Vec<PrivateUserData>>, ApiError> {
    Ok(Json(
        database::users::get_users(&state.db).await.map_http()?,
    ))
}

pub async fn get_private_user_profile(
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
) -> Result<Json<PrivateUserData>, ApiError> {
    Ok(Json(
        database::users::get_user_by_id(&state.db, user_id)
            .await
            .map_http()?
            .into(),
    ))
}

pub async fn delete_user_account(
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
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

    if !is_valid_password {
        Err(AdaJudgeError::Deletion(Deletion::InvalidLoginOrPassword)).map_http()?
    } else {
        database::users::delete_user(&state.db, user_id)
            .await
            .map_http()?;
        Ok(())
    }
}

pub async fn update_user_admin_level(
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
    Json(admin_level): Json<AdminLevel>,
) -> Result<(), ApiError> {
    Ok(
        database::users::change_admin_level(&state.db, user_id, &admin_level)
            .await
            .map_http()?,
    )
}
