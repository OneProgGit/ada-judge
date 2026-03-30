use crate::app_state::AppState;
use crate::crypt::get_password_hash;
use axum::{Json, extract::State, http::StatusCode};
use models::users::RegisterRequest;
use std::sync::Arc;

#[allow(unused)]
pub async fn register(
    State(state): State<Arc<AppState>>,
    Json(user): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<i64>), StatusCode> {
    let password_hash = get_password_hash(&user.password)?;

    Ok((StatusCode::OK, Json(0)))
}
