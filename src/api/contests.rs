use crate::{
    api::ApiError, app_state::AppState, crypt::verify_password, middleware::auth::Auth,
    tools::is_allowed,
};
use aj_models::{
    DeletionRequest,
    contests::{
        ContestPost, ContestPostRequest, ContestRequest, LeaderboardRow, PublicContestConfig,
    },
    errors::AdaJudgeError,
    problems::PublicProblemConfig,
    users::AdminLevel,
};
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use chrono::Utc;
use database::contests::GetContestsMode;
use tools::map::MapHttpExt;

pub async fn get_contest_leaderboard(
    State(state): State<AppState>,
    Path(contest_id): Path<i64>,
) -> Result<Json<Vec<LeaderboardRow>>, ApiError> {
    Ok(Json(
        database::contests::get_leaderboard(&state.db, contest_id)
            .await
            .map_http()?,
    ))
}

pub async fn get_contest_problems(
    State(state): State<AppState>,
    Path(contest_id): Path<i64>,
) -> Result<Json<Vec<PublicProblemConfig>>, ApiError> {
    Ok(Json(
        database::contests::get_problems(&state.db, contest_id)
            .await
            .map_http()?,
    ))
}

pub async fn get_problem_by_id(
    State(state): State<AppState>,
    Path((_, problem_id)): Path<(i64, i64)>,
) -> Result<Json<PublicProblemConfig>, ApiError> {
    Ok(Json(
        database::problems::get_problem(&state.db, problem_id)
            .await
            .map_http()?
            .into(),
    ))
}

pub async fn get_contest_by_id(
    State(state): State<AppState>,
    Auth(auth): Auth,
    Path(contest_id): Path<i64>,
) -> Result<Json<PublicContestConfig>, ApiError> {
    let mut contest: PublicContestConfig = database::contests::get_contest(&state.db, contest_id)
        .await
        .map_http()?
        .into();

    if contest.hidden
        && !is_allowed(auth.id, contest.owner_id, &auth.admin_level)
        && contest.co_authors.binary_search(&auth.id).is_err()
    {
        return Err(AdaJudgeError::Forbidden).map_http()?;
    }

    let now = Utc::now();

    if now < contest.starts_at
        && !is_allowed(auth.id, contest.owner_id, &auth.admin_level)
        && contest.co_authors.binary_search(&auth.id).is_err()
    {
        contest.statements_url_ru = String::default();
        contest.statements_url_en = String::default();
    }
    if now < contest.finishes_at
        && !is_allowed(auth.id, contest.owner_id, &auth.admin_level)
        && contest.co_authors.binary_search(&auth.id).is_err()
    {
        contest.editorial_url_ru = String::default();
        contest.editorial_url_en = String::default();
    }

    Ok(Json(contest))
}

pub async fn get_contests(
    State(state): State<AppState>,
    Auth(auth): Auth,
) -> Result<Json<Vec<PublicContestConfig>>, ApiError> {
    let mode = if auth.admin_level == AdminLevel::Owner {
        GetContestsMode::All
    } else {
        GetContestsMode::NotHidden(auth.id)
    };
    Ok(Json(
        database::contests::get_contests(&state.db, mode)
            .await
            .map_http()?,
    ))
}

pub async fn get_my_contests(
    State(state): State<AppState>,
    Auth(auth): Auth,
) -> Result<Json<Vec<PublicContestConfig>>, ApiError> {
    Ok(Json(
        database::contests::get_contests(&state.db, GetContestsMode::User(auth.id))
            .await
            .map_http()?,
    ))
}

pub async fn create_contest(
    State(state): State<AppState>,
    Auth(auth): Auth,
    Json(request): Json<ContestRequest>,
) -> Result<Json<i64>, ApiError> {
    if request.starts_at >= request.finishes_at {
        Err(StatusCode::BAD_REQUEST)
    } else {
        let mut co_authors = request.co_authors;
        co_authors.sort_unstable();
        let id = database::contests::create_contest(
            &state.db,
            auth.id,
            &request.name_ru,
            &request.name_en,
            &request.starts_at,
            &request.ends_at,
            &request.statements_url_ru,
            &request.editorial_url_ru,
            &request.statements_url_en,
            &request.editorial_url_en,
            request.hidden,
            request.upsolving_opened,
            request.hide_solutions,
            request.hide_leaderboard,
        )
        .await
        .map_http()?;
        database::contests::insert_contest_co_authors(&state.db, id, &co_authors)
            .await
            .map_http()?;
        Ok(Json(id))
    }
}

pub async fn update_contest(
    State(state): State<AppState>,
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

        let mut co_authors = request.co_authors;
        co_authors.sort_unstable();

        database::contests::update_contest(
            &state.db,
            contest_id,
            &request.name_ru,
            &request.name_en,
            &request.starts_at,
            &request.ends_at,
            &request.statements_url_ru,
            &request.editorial_url_ru,
            &request.statements_url_en,
            &request.editorial_url_en,
            request.hidden,
            request.upsolving_opened,
            request.hide_solutions,
            request.hide_leaderboard,
        )
        .await
        .map_http()?;

        database::contests::insert_contest_co_authors(&state.db, contest_id, &co_authors)
            .await
            .map_http()?;

        Ok(())
    }
}

pub async fn delete_contest(
    State(state): State<AppState>,
    Path(contest_id): Path<i64>,
    Auth(auth): Auth,
    Json(request): Json<DeletionRequest>,
) -> Result<(), StatusCode> {
    if request.login != auth.login
        || request.password != request.password_confirmation
        || !request.deletion_confirmation
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    let is_valid_password = verify_password(&auth.password_hash, &request.password).map_http()?;

    if !is_valid_password {
        Err(StatusCode::BAD_REQUEST)
    } else {
        let contest = database::contests::get_contest_by_id(&state.db, contest_id)
            .await
            .map_http()?;
        if !is_allowed(auth.id, contest.owner_id, &auth.admin_level) {
            Err(StatusCode::FORBIDDEN)
        } else {
            database::contests::delete_contest(&state.db, contest_id)
                .await
                .map_http()?;
            Ok(())
        }
    }
}

pub async fn create_contest_post(
    State(state): State<AppState>,
    Path(contest_id): Path<i64>,
    Auth(auth): Auth,
    Json(request): Json<ContestPostRequest>,
) -> Result<Json<i64>, StatusCode> {
    let contest = database::contests::get_contest_by_id(&state.db, contest_id)
        .await
        .map_http()?;
    if !is_allowed(auth.id, contest.owner_id, &auth.admin_level) {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(Json(
        database::contests::create_contest_post(
            &state.db,
            auth.id,
            contest_id,
            &request.title_ru,
            &request.text_ru,
            &request.title_en,
            &request.text_en,
        )
        .await
        .map_http()?,
    ))
}

pub async fn update_contest_post(
    State(state): State<AppState>,
    Path(post_id): Path<i64>,
    Auth(auth): Auth,
    Json(request): Json<ContestPostRequest>,
) -> Result<(), StatusCode> {
    let post = database::contests::get_contest_post(&state.db, post_id)
        .await
        .map_http()?;
    if !is_allowed(auth.id, Some(post.owner_id), &auth.admin_level) {
        return Err(StatusCode::FORBIDDEN);
    }
    database::contests::update_contest_post(
        &state.db,
        post_id,
        &request.title_ru,
        &request.text_ru,
        &request.title_en,
        &request.text_en,
    )
    .await
    .map_http()?;

    Ok(())
}

pub async fn delete_contest_post(
    State(state): State<AppState>,
    Path(post_id): Path<i64>,
    Auth(auth): Auth,
    Json(request): Json<DeletionRequest>,
) -> Result<(), StatusCode> {
    if request.login != auth.login
        || request.password != request.password_confirmation
        || !request.deletion_confirmation
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    let is_valid_password = verify_password(&auth.password_hash, &request.password).map_http()?;

    if !is_valid_password {
        Err(StatusCode::BAD_REQUEST)
    } else {
        let post = database::contests::get_contest_post(&state.db, post_id)
            .await
            .map_http()?;
        if !is_allowed(auth.id, Some(post.owner_id), &auth.admin_level) {
            Err(StatusCode::FORBIDDEN)
        } else {
            database::contests::delete_contest_post(&state.db, post_id)
                .await
                .map_http()?;
            Ok(())
        }
    }
}

pub async fn get_contest_post_by_id(
    State(state): State<AppState>,
    Path(post_id): Path<i64>,
) -> Result<Json<ContestPost>, StatusCode> {
    Ok(Json(
        database::contests::get_contest_post(&state.db, post_id)
            .await
            .map_http()?,
    ))
}

pub async fn get_contest_posts(
    State(state): State<AppState>,
    Auth(auth): Auth,
    Path(contest_id): Path<i64>,
) -> Result<Json<Vec<i64>>, StatusCode> {
    let contest = database::contests::get_contest_by_id(&state.db, contest_id)
        .await
        .map_http()?;

    if contest.hidden && !is_allowed(auth.id, contest.owner_id, &auth.admin_level) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(Json(
        database::contests::get_contest_posts(&state.db, contest_id)
            .await
            .map_http()?,
    ))
}
