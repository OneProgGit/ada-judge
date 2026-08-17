use crate::{app_state::AppState, middleware::auth::Auth, tools::is_allowed};
use aj_models::users::AdminLevel;
use axum::{
    extract::{Path, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use chrono::Utc;
use sqlx::PgPool;

pub async fn ensure_contest_started_common(
    pool: &PgPool,
    user_id: i64,
    contest_id: i64,
    admin_level: AdminLevel,
) -> Result<(), Response> {
    let Ok(contest) = database::contests::get_contest(pool, contest_id).await else {
        return Err(StatusCode::BAD_REQUEST.into_response());
    };

    let now = Utc::now();

    if (now < contest.starts_at || contest.hidden)
        && !is_allowed(user_id, contest.owner_id, &admin_level)
        && contest.co_authors.binary_search(&user_id).is_err()
    {
        Err(StatusCode::FORBIDDEN.into_response())
    } else {
        Ok(())
    }
}

pub async fn ensure_contest_finished_common(
    pool: &PgPool,
    user_id: i64,
    contest_id: i64,
    admin_level: AdminLevel,
) -> Result<(), Response> {
    let Ok(contest) = database::contests::get_contest(pool, contest_id).await else {
        return Err(StatusCode::BAD_REQUEST.into_response());
    };

    let now = Utc::now();

    if (now < contest.finishes_at || contest.hidden)
        && !is_allowed(user_id, contest.owner_id, &admin_level)
        && contest.co_authors.binary_search(&user_id).is_err()
        && contest.leaderboard_hidden
    {
        Err(StatusCode::FORBIDDEN.into_response())
    } else {
        Ok(())
    }
}

pub async fn ensure_contest_started_1(
    Path(contest_id): Path<i64>,
    State(state): State<AppState>,
    Auth(auth): Auth,
    req: Request,
    next: Next,
) -> Response {
    if let Err(e) =
        ensure_contest_started_common(&state.db, auth.id, contest_id, auth.admin_level).await
    {
        return e;
    }

    next.run(req).await
}

pub async fn ensure_contest_started_2(
    Path((contest_id, _)): Path<(i64, i64)>,
    State(state): State<AppState>,
    Auth(auth): Auth,
    req: Request,
    next: Next,
) -> Response {
    if let Err(e) =
        ensure_contest_started_common(&state.db, auth.id, contest_id, auth.admin_level).await
    {
        return e;
    }

    next.run(req).await
}

pub async fn ensure_contest_finished(
    Path(contest_id): Path<i64>,
    State(state): State<AppState>,
    Auth(auth): Auth,
    req: Request,
    next: Next,
) -> Response {
    if let Err(e) =
        ensure_contest_finished_common(&state.db, auth.id, contest_id, auth.admin_level).await
    {
        return e;
    }

    next.run(req).await
}
