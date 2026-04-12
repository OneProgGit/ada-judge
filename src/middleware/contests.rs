use std::sync::Arc;

use crate::{app_state::AppState, middleware::auth::Auth};
use ada_judge_public_models::users::AdminLevel;
use axum::{
    extract::{Path, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use chrono::Utc;
use sqlx::PgPool;

async fn check_contest_started_common(
    pool: &PgPool,
    contest_id: i64,
    admin_level: AdminLevel,
) -> Result<(), Response> {
    let Ok(contest_config) = database::contests::get_contest_by_id(pool, contest_id).await else {
        return Err(StatusCode::BAD_REQUEST.into_response());
    };

    let now = Utc::now();

    if now < contest_config.starts_at && admin_level < AdminLevel::AdminII {
        Err(StatusCode::FORBIDDEN.into_response())
    } else {
        Ok(())
    }
}

pub async fn check_contest_started(
    Path(contest_id): Path<i64>,
    State(state): State<Arc<AppState>>,
    Auth(auth): Auth,
    req: Request,
    next: Next,
) -> Response {
    log::info!("Check contest started");

    if let Err(e) = check_contest_started_common(&state.db, contest_id, auth.admin_level).await {
        return e;
    }

    next.run(req).await
}

pub async fn check_contest_started_2_path_elements(
    Path((contest_id, _)): Path<(i64, i64)>,
    State(state): State<Arc<AppState>>,
    Auth(auth): Auth,
    req: Request,
    next: Next,
) -> Response {
    log::info!("Check contest started");

    if let Err(e) = check_contest_started_common(&state.db, contest_id, auth.admin_level).await {
        return e;
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

    let Ok(contest_config) = database::contests::get_contest_by_id(&state.db, contest_id).await
    else {
        return StatusCode::BAD_REQUEST.into_response();
    };

    let now = Utc::now();

    if (now < contest_config.starts_at || now > contest_config.ends_at)
        && auth.admin_level < AdminLevel::AdminII
    {
        StatusCode::FORBIDDEN.into_response()
    } else {
        next.run(req).await
    }
}

pub async fn check_contest_ended(
    Path(contest_id): Path<i64>,
    State(state): State<Arc<AppState>>,
    Auth(auth): Auth,
    req: Request,
    next: Next,
) -> Response {
    log::info!("Check contest ended");

    let Ok(contest_config) = database::contests::get_contest_by_id(&state.db, contest_id).await
    else {
        return StatusCode::BAD_REQUEST.into_response();
    };

    let now = Utc::now();

    if now <= contest_config.ends_at && auth.admin_level < AdminLevel::AdminII {
        StatusCode::FORBIDDEN.into_response()
    } else {
        next.run(req).await
    }
}
