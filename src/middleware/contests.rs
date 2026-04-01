use std::sync::Arc;

use crate::app_state::AppState;
use axum::{
    extract::{Path, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use chrono::Utc;
use database::get_contest_by_id;

pub async fn check_contest_started(
    Path(contest_id): Path<i64>,
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Response {
    let Ok(contest_config) = get_contest_by_id(&state.db, contest_id).await else {
        return StatusCode::BAD_REQUEST.into_response();
    };

    let now = Utc::now();

    if now < contest_config.starts_at {
        return StatusCode::BAD_REQUEST.into_response();
    }

    next.run(req).await
}

pub async fn check_contest_started_and_not_ended(
    Path(contest_id): Path<i64>,
    State(state): State<Arc<AppState>>,
    req: Request,
    next: Next,
) -> Response {
    let Ok(contest_config) = get_contest_by_id(&state.db, contest_id).await else {
        return StatusCode::BAD_REQUEST.into_response();
    };

    let now = Utc::now();

    if now < contest_config.starts_at || now > contest_config.ends_at {
        return StatusCode::BAD_REQUEST.into_response();
    }

    next.run(req).await
}
