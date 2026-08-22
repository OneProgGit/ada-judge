use std::path::PathBuf;

use crate::{
    api::ApiError, app_state::AppState, crypt::verify_password, middleware::auth::Auth,
    tools::is_allowed,
};
use aj_models::{
    DeletionRequest,
    contests::{
        ContestPost, ContestPostRequest, ContestRequest, LeaderboardRow, PublicContestConfig,
    },
    errors::{AdaJudgeError, Contest, Deletion},
    problems::PublicProblemConfig,
    users::AdminLevel,
};
use axum::{
    Json,
    extract::{Path, State},
};
use chrono::Utc;
use database::contests::GetContestsMode;
use tokio::fs;
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
        .map_http()?;

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
) -> Result<(), ApiError> {
    if request.starts_at >= request.finishes_at {
        Err(AdaJudgeError::Contest(Contest::Time)).map_http()?
    } else {
        let mut request = request;
        request.co_authors.sort_unstable();
        database::contests::create_contest(&state.db, auth.id, &request)
            .await
            .map_http()?;
        Ok(())
    }
}

pub async fn update_contest(
    State(state): State<AppState>,
    Path(contest_id): Path<i64>,
    Auth(auth): Auth,
    Json(request): Json<ContestRequest>,
) -> Result<(), ApiError> {
    if request.starts_at >= request.finishes_at {
        Err(AdaJudgeError::Contest(Contest::Time)).map_http()?
    } else {
        let contest = database::contests::get_contest(&state.db, contest_id)
            .await
            .map_http()?;

        if !is_allowed(auth.id, contest.owner_id, &auth.admin_level) {
            return Err(AdaJudgeError::Forbidden).map_http()?;
        }

        let mut request = request;
        request.co_authors.sort_unstable();

        database::contests::update_contest(&state.db, contest_id, &request)
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
) -> Result<(), ApiError> {
    if request.login != auth.login {
        return Err(AdaJudgeError::Deletion(Deletion::InvalidLoginOrPassword)).map_http()?;
    }
    if !request.deletion_confirmation {
        return Err(AdaJudgeError::Deletion(
            Deletion::MissingDeletionConfirmation,
        ))
        .map_http()?;
    }
    let is_valid_password = verify_password(&auth.password_hash, &request.password).map_http()?;

    if is_valid_password {
        let contest = database::contests::get_contest(&state.db, contest_id)
            .await
            .map_http()?;
        if is_allowed(auth.id, contest.owner_id, &auth.admin_level) {
            let problems = database::contests::get_problems(&state.db, contest_id)
                .await
                .map_http()?;
            for problem in problems {
                let problem_id = problem.id;
                fs::remove_dir_all(PathBuf::from(format!("/problems/{problem_id}")))
                    .await
                    .map_err(|_| AdaJudgeError::Internal)
                    .map_http()?;
                fs::remove_file(PathBuf::from(format!("/problems/{problem_id}.zip")))
                    .await
                    .map_err(|_| AdaJudgeError::Internal)
                    .map_http()?;
            }

            database::contests::delete_contest(&state.db, contest_id)
                .await
                .map_http()?;
            Ok(())
        } else {
            Err(AdaJudgeError::Forbidden).map_http()?
        }
    } else {
        Err(AdaJudgeError::Deletion(Deletion::InvalidLoginOrPassword)).map_http()?
    }
}

pub async fn create_contest_post(
    State(state): State<AppState>,
    Path(contest_id): Path<i64>,
    Auth(auth): Auth,
    Json(request): Json<ContestPostRequest>,
) -> Result<(), ApiError> {
    let contest = database::contests::get_contest(&state.db, contest_id)
        .await
        .map_http()?;
    if !is_allowed(auth.id, contest.owner_id, &auth.admin_level) {
        return Err(AdaJudgeError::Forbidden).map_http()?;
    }
    database::contests::create_contest_post(&state.db, auth.id, contest_id, &request)
        .await
        .map_http()?;
    Ok(())
}

pub async fn update_contest_post(
    State(state): State<AppState>,
    Path(post_id): Path<i64>,
    Auth(auth): Auth,
    Json(request): Json<ContestPostRequest>,
) -> Result<(), ApiError> {
    let post = database::contests::get_contest_post(&state.db, post_id)
        .await
        .map_http()?;
    if !is_allowed(auth.id, Some(post.owner_id), &auth.admin_level) {
        return Err(AdaJudgeError::Forbidden).map_http()?;
    }
    database::contests::update_contest_post(&state.db, post_id, &request)
        .await
        .map_http()?;

    Ok(())
}

pub async fn delete_contest_post(
    State(state): State<AppState>,
    Path(post_id): Path<i64>,
    Auth(auth): Auth,
    Json(request): Json<DeletionRequest>,
) -> Result<(), ApiError> {
    if request.login != auth.login {
        return Err(AdaJudgeError::Deletion(Deletion::InvalidLoginOrPassword)).map_http()?;
    }
    if !request.deletion_confirmation {
        return Err(AdaJudgeError::Deletion(
            Deletion::MissingDeletionConfirmation,
        ))
        .map_http()?;
    }
    let is_valid_password = verify_password(&auth.password_hash, &request.password).map_http()?;

    if is_valid_password {
        let post = database::contests::get_contest_post(&state.db, post_id)
            .await
            .map_http()?;
        if is_allowed(auth.id, Some(post.owner_id), &auth.admin_level) {
            database::contests::delete_contest_post(&state.db, post_id)
                .await
                .map_http()?;
            Ok(())
        } else {
            Err(AdaJudgeError::Forbidden).map_http()?
        }
    } else {
        Err(AdaJudgeError::Deletion(Deletion::InvalidLoginOrPassword)).map_http()?
    }
}

pub async fn get_contest_post_by_id(
    State(state): State<AppState>,
    Path(post_id): Path<i64>,
) -> Result<Json<ContestPost>, ApiError> {
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
) -> Result<Json<Vec<ContestPost>>, ApiError> {
    let contest = database::contests::get_contest(&state.db, contest_id)
        .await
        .map_http()?;

    if contest.hidden && !is_allowed(auth.id, contest.owner_id, &auth.admin_level) {
        return Err(AdaJudgeError::Forbidden).map_http()?;
    }

    Ok(Json(
        database::contests::get_contest_posts(&state.db, contest_id)
            .await
            .map_http()?,
    ))
}
