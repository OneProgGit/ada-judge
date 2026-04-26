use std::sync::Arc;

use crate::{app_state::AppState, middleware::auth::Auth, tools::is_allowed};
use ada_judge_public_models::{
    contests::{ContestRequest, LeaderboardRow, PublicContestConfig},
    problems::PublicProblemConfig,
    users::AdminLevel,
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::Utc;
use tools::map::MapHttpExt;

pub async fn get_contest_leaderboard(
    State(state): State<Arc<AppState>>,
    Path(contest_id): Path<i64>,
) -> Result<Json<Vec<LeaderboardRow>>, StatusCode> {
    Ok(Json(
        database::contests::get_contest_leaderboard(&state.db, contest_id)
            .await
            .map_http()?,
    ))
}

pub async fn get_contest_problems(
    State(state): State<Arc<AppState>>,
    Path(contest_id): Path<i64>,
) -> Result<Json<Vec<i64>>, StatusCode> {
    Ok(Json(
        database::contests::get_contest_problems(&state.db, contest_id)
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

pub async fn get_contest_by_id(
    State(state): State<Arc<AppState>>,
    Auth(auth): Auth,
    Path(contest_id): Path<i64>,
) -> Result<Json<PublicContestConfig>, StatusCode> {
    let mut contest: PublicContestConfig =
        database::contests::get_contest_by_id(&state.db, contest_id)
            .await
            .map_http()?
            .into();

    let now = Utc::now();

    if now < contest.starts_at {
        if contest.owner_id.is_none() && auth.admin_level != AdminLevel::Owner {
            contest.statements_url = String::default();
        } else if let Some(owner_id) = contest.owner_id
            && owner_id != auth.id
            && auth.admin_level != AdminLevel::Owner
        {
            contest.statements_url = String::default();
        }
    }

    Ok(Json(contest))
}

pub async fn get_contests(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<i64>>, StatusCode> {
    Ok(Json(
        database::contests::get_contests(&state.db)
            .await
            .map_http()?,
    ))
}

pub async fn create_contest(
    State(state): State<Arc<AppState>>,
    Auth(auth): Auth,
    Json(request): Json<ContestRequest>,
) -> Result<Json<i64>, StatusCode> {
    if request.starts_at >= request.ends_at {
        Err(StatusCode::BAD_REQUEST)
    } else {
        Ok(Json(
            database::contests::create_contest(
                &state.db,
                auth.id,
                &request.name,
                &request.starts_at,
                &request.ends_at,
            )
            .await
            .map_http()?,
        ))
    }
}

pub async fn update_contest(
    State(state): State<Arc<AppState>>,
    Path(contest_id): Path<i64>,
    Auth(auth): Auth,
    Json(request): Json<ContestRequest>,
) -> Result<(), StatusCode> {
    if request.starts_at >= request.ends_at {
        Err(StatusCode::BAD_REQUEST)
    } else {
        let contest = database::contests::get_contest_by_id(&state.db, contest_id)
            .await
            .map_http()?;

        if !is_allowed(auth.id, contest.owner_id, &auth.admin_level) {
            return Err(StatusCode::FORBIDDEN);
        }

        database::contests::update_contest(
            &state.db,
            contest_id,
            &request.name,
            &request.starts_at,
            &request.ends_at,
        )
        .await
        .map_http()?;
        Ok(())
    }
}
