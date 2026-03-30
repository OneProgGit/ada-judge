use axum::{Json, extract::State, response::IntoResponse};
use models::users::RegisterRequest;

use crate::app_state::AppState;

pub async fn register(
    State(state): State<AppState>,
    Json(user): Json<RegisterRequest>,
) -> impl IntoResponse {
}
