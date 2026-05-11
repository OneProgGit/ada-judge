use crate::{app_state::AppState, middleware::auth::Auth};
use axum::{Json, extract::State, http::StatusCode};
use database::problems::get_all_user_problems;
use tools::map::MapHttpExt;

pub async fn get_problems(State(state): State<AppState>) -> Result<Json<Vec<i64>>, StatusCode> {
    Ok(Json(get_all_user_problems(&state.db, -1).await.map_http()?))
}

pub async fn get_my_problems(
    State(state): State<AppState>,
    Auth(auth): Auth,
) -> Result<Json<Vec<i64>>, StatusCode> {
    Ok(Json(
        get_all_user_problems(&state.db, auth.id).await.map_http()?,
    ))
}
