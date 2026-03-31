use std::sync::Arc;

use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use models::contests::{GetContestLeaderboardRequest, LeaderboardRow};
use tools::map::MapHttpExt;

use crate::app_state::AppState;

pub async fn get_contest_leaderboard(
    State(state): State<Arc<AppState>>,
    Query(request): Query<GetContestLeaderboardRequest>,
) -> Result<Json<Vec<LeaderboardRow>>, StatusCode> {
    Ok(Json(
        database::get_contest_leaderboard(&state.db, request.contest_id)
            .await
            .map_http()?,
    ))
}
