use crate::api::ApiError;
use crate::tools::{is_allowed, is_contest_active};
use crate::{app_state::AppState, middleware::auth::Auth};
use aj_models::errors::AdaJudgeError;
use aj_models::testing::{Submission, SubmissonRequest};
use apalis::prelude::TaskSink;
use axum::body::Body;
use axum::http::header;
use axum::response::IntoResponse;
use axum::{
    Json,
    body::Bytes,
    extract::{Multipart, Path, State},
};
use database::tools::MapDbExt;
use models::testing::SubmissionTask;
use std::path::PathBuf;
use tokio::process::Command;
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};
use tokio_util::io::ReaderStream;
use tools::map::MapHttpExt;

pub async fn get_my_problem_submissions(
    State(state): State<AppState>,
    Path(problem_id): Path<i64>,
    Auth(auth): Auth,
) -> Result<Json<Vec<Submission>>, ApiError> {
    Ok(Json(
        database::submissions::get_problem_submissions(&state.db, Some(auth.id), problem_id)
            .await
            .map_http()?,
    ))
}

pub async fn get_problem_submissions(
    State(state): State<AppState>,
    Path(problem_id): Path<i64>,
    Auth(auth): Auth,
) -> Result<Json<Vec<Submission>>, ApiError> {
    let problem = database::problems::get_problem(&state.db, problem_id)
        .await
        .map_http()?;
    let contest = database::contests::get_contest(&state.db, problem.contest_id)
        .await
        .map_http()?;
    if !is_allowed(auth.id, problem.owner_id, &auth.admin_level)
        && !is_allowed(auth.id, contest.owner_id, &auth.admin_level)
        && contest.co_authors.binary_search(&auth.id).is_err()
    {
        return Err(AdaJudgeError::Forbidden).map_http()?;
    }

    Ok(Json(
        database::submissions::get_problem_submissions(&state.db, None, problem_id)
            .await
            .map_http()?,
    ))
}

pub async fn get_submission(
    State(state): State<AppState>,
    Path(submission_id): Path<i64>,
    Auth(auth): Auth,
) -> Result<Json<Submission>, ApiError> {
    let submission = database::submissions::get_submission(&state.db, submission_id)
        .await
        .map_http()?;
    let problem = database::problems::get_problem(&state.db, submission.problem_id)
        .await
        .map_http()?;
    let contest = database::contests::get_contest(&state.db, problem.contest_id)
        .await
        .map_http()?;
    if is_allowed(auth.id, Some(submission.user_id), &auth.admin_level)
        || is_allowed(auth.id, problem.owner_id, &auth.admin_level)
        || is_allowed(auth.id, contest.owner_id, &auth.admin_level)
        || contest.co_authors.binary_search(&auth.id).is_ok()
    {
        Ok(Json(submission))
    } else {
        Err(AdaJudgeError::Forbidden).map_http()?
    }
}

pub async fn download_submission(
    State(state): State<AppState>,
    Path(submission_id): Path<i64>,
    Auth(auth): Auth,
) -> Result<impl IntoResponse, ApiError> {
    let submission = database::submissions::get_submission(&state.db, submission_id)
        .await
        .map_http()?;
    let problem = database::problems::get_problem(&state.db, submission.problem_id)
        .await
        .map_http()?;
    let contest = database::contests::get_contest(&state.db, problem.contest_id)
        .await
        .map_http()?;

    if (!is_allowed(auth.id, Some(submission.user_id), &auth.admin_level)
        || contest.solutions_hidden)
        && !is_allowed(auth.id, problem.owner_id, &auth.admin_level)
        && !is_allowed(auth.id, contest.owner_id, &auth.admin_level)
        && contest.co_authors.binary_search(&auth.id).is_err()
    {
        Err(AdaJudgeError::Forbidden).map_http()?
    } else {
        let file_ext = &submission.language.file_ext();
        let file_path = PathBuf::from(format!("/submissions_envs/{submission_id}/run.{file_ext}"));
        let file = File::open(&file_path)
            .await
            .map_err(|_| AdaJudgeError::Internal)
            .map_http()?;
        let stream = ReaderStream::new(file);
        let body = Body::from_stream(stream);

        let headers = [
            (header::CONTENT_TYPE, "application/octet-stream".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"solution.{file_ext}\""),
            ),
        ];

        Ok((headers, body))
    }
}

#[allow(clippy::too_many_lines)]
pub async fn submit(
    State(state): State<AppState>,
    Auth(auth): Auth,
    Path((contest_id, problem_id)): Path<(i64, i64)>,
    mut multipart: Multipart,
) -> Result<Json<i64>, ApiError> {
    let mut submission: Option<SubmissonRequest> = None;
    let mut file_stream: Option<Bytes> = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| AdaJudgeError::BadRequest)
        .map_http()?
    {
        match field.name() {
            Some("submission_data") => {
                let text = field
                    .text()
                    .await
                    .map_err(|_| AdaJudgeError::BadRequest)
                    .map_http()?;
                submission = Some(
                    serde_json::from_str(&text)
                        .map_err(|_| AdaJudgeError::BadRequest)
                        .map_http()?,
                );
            }
            Some("submission_file") => {
                file_stream = Some(
                    field
                        .bytes()
                        .await
                        .map_err(|_| AdaJudgeError::Internal)
                        .map_http()?,
                );
            }
            _ => {}
        }
    }

    let Some(submission) = submission else {
        return Err(AdaJudgeError::BadRequest).map_http()?;
    };

    let Some(file_stream) = file_stream else {
        return Err(AdaJudgeError::BadRequest).map_http()?;
    };

    if !is_contest_active(&state.db, auth.id, contest_id, problem_id, auth.admin_level).await {
        return Err(AdaJudgeError::Forbidden).map_http()?;
    }
    let submission_id = database::submissions::create_submission(
        &state.db,
        auth.id,
        problem_id,
        &submission.language,
    )
    .await
    .map_http()?;
    let run_dir = PathBuf::from("/submissions_envs").join(submission_id.to_string());
    fs::create_dir(run_dir.clone())
        .await
        .map_err(|_| AdaJudgeError::Internal)
        .map_db(&state.db, submission_id, None)
        .await
        .map_err(|_| AdaJudgeError::Internal)
        .map_http()?;
    let run_path = run_dir.join(format!("run.{}", submission.language.file_ext()));
    let mut run_file = File::create(run_path)
        .await
        .map_err(|_| AdaJudgeError::Internal)
        .map_db(&state.db, submission_id, None)
        .await
        .map_err(|_| AdaJudgeError::Internal)
        .map_http()?;
    run_file
        .write_all(&file_stream)
        .await
        .map_err(|_| AdaJudgeError::Internal)
        .map_db(&state.db, submission_id, None)
        .await
        .map_err(|_| AdaJudgeError::Internal)
        .map_http()?;
    run_file
        .flush()
        .await
        .map_err(|_| AdaJudgeError::Internal)
        .map_http()?;
    let status = Command::new("chown")
        .args([
            "-R",
            "1000:1000",
            run_dir.to_str().ok_or(AdaJudgeError::Internal).map_http()?,
        ])
        .status()
        .await
        .map_err(|_| AdaJudgeError::Internal)
        .map_http()?;
    if status.code().is_some_and(|code| code != 0) {
        return Err(AdaJudgeError::Internal).map_http()?;
    }

    let submission_task = SubmissionTask {
        problem_path: PathBuf::from("/problems").join(problem_id.to_string()),
        problem_id,
        id: submission_id,
        run_dir,
        language: submission.language,
    };

    state
        .apalis_backend
        .lock()
        .await
        .push(submission_task)
        .await
        .map_err(|_| AdaJudgeError::Internal)
        .map_db(&state.db, submission_id, None)
        .await
        .map_err(|_| AdaJudgeError::Internal)
        .map_http()?;

    Ok(Json(submission_id))
}

pub async fn retest_problem_submissions(
    State(state): State<AppState>,
    Auth(auth): Auth,
    Path(problem_id): Path<i64>,
) -> Result<(), ApiError> {
    let problem = database::problems::get_problem(&state.db, problem_id)
        .await
        .map_http()?;
    let contest = database::contests::get_contest(&state.db, problem.contest_id)
        .await
        .map_http()?;

    if !is_allowed(auth.id, contest.owner_id, &auth.admin_level)
        && !is_allowed(auth.id, problem.owner_id, &auth.admin_level)
    {
        return Err(AdaJudgeError::Forbidden).map_http()?;
    }

    database::submissions::delete_problem_subgroups_results(&state.db, problem_id)
        .await
        .map_http()?;
    database::submissions::delete_problem_tests_results(&state.db, problem_id)
        .await
        .map_http()?;
    database::submissions::make_submissions_pending(&state.db, problem_id)
        .await
        .map_http()?;

    for submission in database::submissions::get_problem_submissions(&state.db, None, problem_id)
        .await
        .map_http()?
    {
        let run_dir = PathBuf::from("/submissions_envs").join(submission.id.to_string());
        let submission_task = SubmissionTask {
            problem_path: PathBuf::from("/problems").join(submission.problem_id.to_string()),
            problem_id: submission.problem_id,
            id: submission.id,
            run_dir,
            language: submission.language,
        };
        state
            .apalis_backend
            .lock()
            .await
            .push(submission_task)
            .await
            .map_err(|_| AdaJudgeError::Internal)
            .map_db(&state.db, submission.id, None)
            .await
            .map_err(|_| AdaJudgeError::Forbidden)
            .map_http()?;
    }
    Ok(())
}
