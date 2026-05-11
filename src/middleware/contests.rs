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

pub async fn check_contest_started_common(
    pool: &PgPool,
    user_id: i64,
    contest_id: i64,
    admin_level: AdminLevel,
) -> Result<(), Response> {
    let Ok(contest) = database::contests::get_contest_by_id(pool, contest_id).await else {
        return Err(StatusCode::BAD_REQUEST.into_response());
    };

    let now = Utc::now();

    if now < contest.starts_at {
        if contest.owner_id.is_none() && admin_level != AdminLevel::Owner {
            Err(StatusCode::FORBIDDEN.into_response())
        } else if let Some(owner_id) = contest.owner_id
            && owner_id != user_id
            && admin_level != AdminLevel::Owner
        {
            Err(StatusCode::FORBIDDEN.into_response())
        } else {
            Ok(())
        }
    } else {
        Ok(())
    }
}

pub async fn check_contest_ended_common(
    pool: &PgPool,
    user_id: i64,
    contest_id: i64,
    admin_level: AdminLevel,
) -> Result<(), Response> {
    let Ok(contest) = database::contests::get_contest_by_id(pool, contest_id).await else {
        return Err(StatusCode::BAD_REQUEST.into_response());
    };

    let now = Utc::now();

    if now < contest.ends_at {
        if contest.owner_id.is_none() && admin_level != AdminLevel::Owner {
            Err(StatusCode::FORBIDDEN.into_response())
        } else if let Some(owner_id) = contest.owner_id
            && owner_id != user_id
            && admin_level != AdminLevel::Owner
        {
            Err(StatusCode::FORBIDDEN.into_response())
        } else {
            Ok(())
        }
    } else {
        Ok(())
    }
}

pub async fn check_contest_started_and_not_ended_common(
    pool: &PgPool,
    user_id: i64,
    contest_id: i64,
    admin_level: AdminLevel,
) -> Result<(), Response> {
    let Ok(contest) = database::contests::get_contest_by_id(pool, contest_id).await else {
        return Err(StatusCode::BAD_REQUEST.into_response());
    };

    let now = Utc::now();

    if now < contest.starts_at || now >= contest.ends_at {
        if contest.owner_id.is_none() && admin_level != AdminLevel::Owner {
            Err(StatusCode::FORBIDDEN.into_response())
        } else if let Some(owner_id) = contest.owner_id
            && owner_id != user_id
            && admin_level != AdminLevel::Owner
        {
            Err(StatusCode::FORBIDDEN.into_response())
        } else {
            Ok(())
        }
    } else {
        Ok(())
    }
}

pub async fn check_contest_started(
    Path(contest_id): Path<i64>,
    State(state): State<AppState>,
    Auth(auth): Auth,
    req: Request,
    next: Next,
) -> Response {
    log::info!("Check contest started");

    if let Err(e) =
        check_contest_started_common(&state.db, auth.id, contest_id, auth.admin_level).await
    {
        return e;
    }

    next.run(req).await
}

pub async fn check_contest_started_2_path_elements(
    Path((contest_id, _)): Path<(i64, i64)>,
    State(state): State<AppState>,
    Auth(auth): Auth,
    req: Request,
    next: Next,
) -> Response {
    log::info!("Check contest started");

    if let Err(e) =
        check_contest_started_common(&state.db, auth.id, contest_id, auth.admin_level).await
    {
        return e;
    }

    next.run(req).await
}

pub async fn check_contest_started_and_not_ended(
    Path(contest_id): Path<i64>,
    State(state): State<AppState>,
    Auth(auth): Auth,
    req: Request,
    next: Next,
) -> Response {
    log::info!("Check contest started and not ended");

    if let Err(e) =
        check_contest_started_and_not_ended_common(&state.db, auth.id, contest_id, auth.admin_level)
            .await
    {
        return e;
    }

    next.run(req).await
}

pub async fn check_contest_ended(
    Path(contest_id): Path<i64>,
    State(state): State<AppState>,
    Auth(auth): Auth,
    req: Request,
    next: Next,
) -> Response {
    log::info!("Check contest ended");

    if let Err(e) =
        check_contest_ended_common(&state.db, auth.id, contest_id, auth.admin_level).await
    {
        return e;
    }

    next.run(req).await
}
