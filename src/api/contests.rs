use std::sync::Arc;

use crate::app_state::AppState;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use models::{
    contest_config::PublicContestConfig, contests::LeaderboardRow,
    problem_config::PublicProblemConfig,
};
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
) -> Result<Json<Vec<PublicProblemConfig>>, StatusCode> {
    Ok(Json(
        database::get_contest_public_problems(&state.db, contest_id)
            .await
            .map_http()?,
    ))
}

pub async fn get_contests(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<PublicContestConfig>>, StatusCode> {
    Ok(Json(
        database::get_contests(&state.db)
            .await
            .map_http()?
            .iter()
            .map(Into::into)
            .collect(),
    ))
}
