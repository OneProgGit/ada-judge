use std::sync::Arc;

use crate::app_state::AppState;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use models::{contests::LeaderboardRow, problem_config::PublicProblemConfig};
use tools::map::MapHttpExt;

pub async fn get_contest_leaderboard(
    State(state): State<Arc<AppState>>,
    Path(contest_id): Path<i64>,
) -> Result<Json<Vec<LeaderboardRow>>, StatusCode> {
    Ok(Json(
        database::get_contest_leaderboard(&state.db, contest_id)
            .await
            .map_http()?,
    ))
}

pub async fn get_contest_problems(
    State(state): State<Arc<AppState>>,
    Path(contest_id): Path<i64>,
) -> Result<Json<Vec<i64>>, StatusCode> {
    Ok(Json(
        database::get_contest_problems(&state.db, contest_id)
            .await
            .map_http()?,
    ))
}

pub async fn get_problem_by_id(
    State(state): State<Arc<AppState>>,
    Path((_, problem_index)): Path<(i64, i64)>,
) -> Result<Json<PublicProblemConfig>, StatusCode> {
    Ok(Json(
        database::get_problem_by_id(&state.db, problem_index)
            .await
            .map_http()?
            .into(),
    ))
}

pub async fn get_problem_by_index_in_contest(
    State(state): State<Arc<AppState>>,
    Path((contest_id, problem_index)): Path<(i64, i64)>,
) -> Result<Json<PublicProblemConfig>, StatusCode> {
    Ok(Json(
        database::get_problem_by_index_in_contest(&state.db, contest_id, problem_index)
            .await
            .map_http()?
            .into(),
    ))
}

pub async fn get_contests(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<i64>>, StatusCode> {
    Ok(Json(database::get_contests(&state.db).await.map_http()?))
}
