use std::sync::Arc;

use crate::app_state::AppState;
use axum::{Json, extract::State, http::StatusCode};
use models::users::RegisterRequest;

#[allow(unused)]
pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(user): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<i64>), StatusCode> {
    Ok((StatusCode::OK, Json(0)))
}
