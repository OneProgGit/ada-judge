use std::sync::Arc;

use crate::{app_state::AppState, middleware::auth::Auth};
use axum::{
    extract::{Path, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use chrono::Utc;
use database::get_contest_by_id;
use models::users::AdminLevel;

pub async fn check_contest_started(
    Path(contest_id): Path<i64>,
    State(state): State<Arc<AppState>>,
    Auth(auth): Auth,
    req: Request,
    next: Next,
) -> Response {
    log::info!("Check contest started");

    let Ok(contest_config) = get_contest_by_id(&state.db, contest_id).await else {
        return StatusCode::BAD_REQUEST.into_response();
    };

    let now = Utc::now();

    if now < contest_config.starts_at && auth.admin_level < AdminLevel::AdminII {
        return StatusCode::BAD_REQUEST.into_response();
    }

    next.run(req).await
}

pub async fn check_contest_started_and_not_ended(
    Path(contest_id): Path<i64>,
    State(state): State<Arc<AppState>>,
    Auth(auth): Auth,
    req: Request,
    next: Next,
) -> Response {
    log::info!("Check contest started and not ended");

    let Ok(contest_config) = get_contest_by_id(&state.db, contest_id).await else {
        return StatusCode::BAD_REQUEST.into_response();
    };

    let now = Utc::now();

    if (now < contest_config.starts_at || now > contest_config.ends_at)
        && auth.admin_level < AdminLevel::AdminII
    {
        return StatusCode::BAD_REQUEST.into_response();
    }

    next.run(req).await
}

pub async fn check_contest_ended(
    Path(contest_id): Path<i64>,
    State(state): State<Arc<AppState>>,
    Auth(auth): Auth,
    req: Request,
    next: Next,
) -> Response {
    log::info!("Check contest ended");

    let Ok(contest_config) = get_contest_by_id(&state.db, contest_id).await else {
        return StatusCode::BAD_REQUEST.into_response();
    };

    let now = Utc::now();

    if now <= contest_config.ends_at && auth.admin_level < AdminLevel::AdminII {
        return StatusCode::BAD_REQUEST.into_response();
    }

    next.run(req).await
}
