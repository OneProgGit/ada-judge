use crate::{app_state::AppState, middleware::auth::Auth};
use ada_judge_public_models::users::{AdminLevel, PrivateUserData, PublicUserData};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use std::sync::Arc;
use tools::map::MapHttpExt;

pub async fn get_public_user_profile(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<i64>,
) -> Result<Json<PublicUserData>, StatusCode> {
    Ok(Json(
        database::users::get_user_by_id(&state.db, user_id)
            .await
            .map_http()?
            .into(),
    ))
}

pub async fn get_my_user_profile(
    State(state): State<Arc<AppState>>,
    Auth(auth): Auth,
) -> Result<Json<PrivateUserData>, StatusCode> {
    Ok(Json(
        database::users::get_user_by_id(&state.db, auth.id)
            .await
            .map_http()?
            .into(),
    ))
}

pub async fn get_users(State(state): State<Arc<AppState>>) -> Result<Json<Vec<i64>>, StatusCode> {
    Ok(Json(
        database::users::get_users(&state.db).await.map_http()?,
    ))
}

pub async fn get_private_user_profile(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<i64>,
) -> Result<Json<PrivateUserData>, StatusCode> {
    Ok(Json(
        database::users::get_user_by_id(&state.db, user_id)
            .await
            .map_http()?
            .into(),
    ))
}

pub async fn delete_user_account(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<i64>,
) -> Result<(), StatusCode> {
    database::users::delete_user(&state.db, user_id)
        .await
        .map_http()?;
    Ok(())
}

pub async fn change_user_admin_level(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<i64>,
    Json(admin_level): Json<AdminLevel>,
) -> Result<(), StatusCode> {
    Ok(
        database::users::change_user_admin_level(&state.db, user_id, &admin_level)
            .await
            .map_http()?,
    )
}
