use std::sync::Arc;

use crate::{app_state::AppState, middleware::auth::Auth};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use models::users::{PrivateUserData, PublicUserData};
use tools::map::MapHttpExt;

pub async fn get_public_user_profile(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<i64>,
) -> Result<Json<PublicUserData>, StatusCode> {
    Ok(Json(
        database::auth::get_user_by_id(&state.db, user_id)
            .await
            .map_http()?
            .into(),
    ))
}

pub async fn get_private_user_profile(
    State(state): State<Arc<AppState>>,
    Auth(auth): Auth,
) -> Result<Json<PrivateUserData>, StatusCode> {
    Ok(Json(
        database::auth::get_user_by_id(&state.db, auth.id)
            .await
            .map_http()?
            .into(),
    ))
}
