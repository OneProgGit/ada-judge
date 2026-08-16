use crate::tools::{is_allowed, is_contest_active};
use crate::{app_state::AppState, middleware::auth::Auth};
use aj_models::testing::get_language_file_extension;
use aj_models::{
    testing::{Submission, SubmissonRequest},
    verdicts::TestingVerdict,
};
use apalis::prelude::TaskSink;
use axum::body::Body;
use axum::http::header;
use axum::response::IntoResponse;
use axum::{
    Json,
    body::Bytes,
    extract::{Multipart, Path, State},
    http::StatusCode,
};
use database::tools::MapDbExt;
use models::testing::SubmissionTask;
use std::path::PathBuf;
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};
use tokio_util::io::ReaderStream;
use tools::map::{MapHttpExt, MapLogExt};

pub async fn get_all_my_submissions(
    State(state): State<AppState>,
    Auth(auth): Auth,
) -> Result<Json<Vec<i64>>, StatusCode> {
    Ok(Json(
        database::submissions::get_all_user_submissions(&state.db, Some(auth.id))
            .await
            .map_http()?,
    ))
}

pub async fn get_my_contest_submissions(
    State(state): State<AppState>,
    Path(contest_id): Path<i64>,
    Auth(auth): Auth,
) -> Result<Json<Vec<i64>>, StatusCode> {
    Ok(Json(
        database::submissions::get_user_contest_submissions(&state.db, Some(auth.id), contest_id)
            .await
            .map_http()?,
    ))
}

pub async fn get_my_problem_submissions(
    State(state): State<AppState>,
    Path(problem_id): Path<i64>,
    Auth(auth): Auth,
) -> Result<Json<Vec<i64>>, StatusCode> {
    Ok(Json(
        database::submissions::get_user_problem_submissions(&state.db, Some(auth.id), problem_id)
            .await
            .map_http()?,
    ))
}

pub async fn get_all_user_submissions(
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
) -> Result<Json<Vec<i64>>, StatusCode> {
    Ok(Json(
        database::submissions::get_all_user_submissions(&state.db, Some(user_id))
            .await
            .map_http()?,
    ))
}

pub async fn get_user_contest_submissions(
    State(state): State<AppState>,
    Path((contest_id, user_id)): Path<(i64, i64)>,
    Auth(auth): Auth,
) -> Result<Json<Vec<i64>>, StatusCode> {
    let contest = database::contests::get_contest_by_id(&state.db, contest_id)
        .await
        .map_http()?;
    if !is_allowed(auth.id, contest.owner_id, &auth.admin_level) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(Json(
        database::submissions::get_user_contest_submissions(&state.db, Some(user_id), contest_id)
            .await
            .map_http()?,
    ))
}

pub async fn get_user_problem_submissions(
    State(state): State<AppState>,
    Path((problem_id, user_id)): Path<(i64, i64)>,
    Auth(auth): Auth,
) -> Result<Json<Vec<i64>>, StatusCode> {
    let problem = database::problems::get_problem(&state.db, problem_id)
        .await
        .map_http()?;
    let contest = database::contests::get_contest_by_id(&state.db, problem.contest_id)
        .await
        .map_http()?;
    if !is_allowed(auth.id, contest.owner_id, &auth.admin_level)
        && !is_allowed(auth.id, problem.owner_id, &auth.admin_level)
    {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(Json(
        database::submissions::get_user_problem_submissions(&state.db, Some(user_id), problem_id)
            .await
            .map_http()?,
    ))
}

pub async fn get_all_submissions(
    State(state): State<AppState>,
) -> Result<Json<Vec<i64>>, StatusCode> {
    Ok(Json(
        database::submissions::get_all_user_submissions(&state.db, None)
            .await
            .map_http()?,
    ))
}

pub async fn get_contest_submissions(
    State(state): State<AppState>,
    Path(contest_id): Path<i64>,
    Auth(auth): Auth,
) -> Result<Json<Vec<i64>>, StatusCode> {
    let contest = database::contests::get_contest_by_id(&state.db, contest_id)
        .await
        .map_http()?;
    if !is_allowed(auth.id, contest.owner_id, &auth.admin_level) {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(Json(
        database::submissions::get_user_contest_submissions(&state.db, None, contest_id)
            .await
            .map_http()?,
    ))
}

pub async fn get_problem_submissions(
    State(state): State<AppState>,
    Path(problem_id): Path<i64>,
    Auth(auth): Auth,
) -> Result<Json<Vec<i64>>, StatusCode> {
    let problem = database::problems::get_problem(&state.db, problem_id)
        .await
        .map_http()?;
    let contest = database::contests::get_contest_by_id(&state.db, problem.contest_id)
        .await
        .map_http()?;
    if !is_allowed(auth.id, contest.owner_id, &auth.admin_level)
        && !is_allowed(auth.id, problem.owner_id, &auth.admin_level)
    {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(Json(
        database::submissions::get_user_problem_submissions(&state.db, None, problem_id)
            .await
            .map_http()?,
    ))
}

pub async fn get_submission(
    State(state): State<AppState>,
    Path(submission_id): Path<i64>,
    Auth(auth): Auth,
) -> Result<Json<Submission>, StatusCode> {
    let submission: Submission = database::submissions::get_submission(&state.db, submission_id)
        .await
        .map_http()?
        .into();
    if !is_allowed(auth.id, Some(submission.user_id), &auth.admin_level) {
        Err(StatusCode::FORBIDDEN)
    } else {
        Ok(Json(submission))
    }
}

pub async fn download_submission(
    State(state): State<AppState>,
    Path(submission_id): Path<i64>,
    Auth(auth): Auth,
) -> Result<impl IntoResponse, StatusCode> {
    let submission: Submission = database::submissions::get_submission(&state.db, submission_id)
        .await
        .map_http()?
        .into();
    let problem = database::problems::get_problem(&state.db, submission.problem_id)
        .await
        .map_http()?;
    let contest = database::contests::get_contest_by_id(&state.db, problem.contest_id)
        .await
        .map_http()?;

    if (!is_allowed(auth.id, Some(submission.user_id), &auth.admin_level) || contest.hide_solutions)
        && !is_allowed(auth.id, problem.owner_id, &auth.admin_level)
        && !is_allowed(auth.id, contest.owner_id, &auth.admin_level)
    {
        Err(StatusCode::FORBIDDEN)
    } else {
        let file_ext = get_language_file_extension(&submission.language);
        let file_path = PathBuf::from(format!(
            "/submissions_envs/{submission_id}/run.{}",
            file_ext
        ));
        let file = File::open(&file_path)
            .await
            .map_log(TestingVerdict::Bug)
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

pub async fn submit(
    State(state): State<AppState>,
    Auth(auth): Auth,
    mut multipart: Multipart,
) -> Result<Json<i64>, StatusCode> {
    let mut submission: Option<SubmissonRequest> = None;
    let mut file_stream: Option<Bytes> = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_log(TestingVerdict::InvalidRequest)
        .map_http()?
    {
        match field.name() {
            Some("submission_data") => {
                let text = field
                    .text()
                    .await
                    .map_log(TestingVerdict::InvalidRequest)
                    .map_http()?;
                submission = Some(
                    serde_json::from_str(&text)
                        .map_log(TestingVerdict::InvalidRequest)
                        .map_http()?,
                );
            }
            Some("submission_file") => {
                file_stream = Some(
                    field
                        .bytes()
                        .await
                        .map_log(TestingVerdict::Bug)
                        .map_http()?,
                );
            }
            _ => {}
        }
    }

    let Some(submission) = submission else {
        return Err(StatusCode::BAD_REQUEST);
    };

    let Some(file_stream) = file_stream else {
        return Err(StatusCode::BAD_REQUEST);
    };

    let problem = database::problems::get_problem(&state.db, submission.problem_id)
        .await
        .map_http()?;

    if !is_contest_active(
        &state.db,
        auth.id,
        problem.contest_id,
        problem.id,
        auth.admin_level,
    )
    .await
    {
        return Err(StatusCode::FORBIDDEN);
    }
    let submission_id = database::submissions::insert_submission(
        &state.db,
        auth.id,
        submission.problem_id,
        &submission.language,
    )
    .await
    .map_http()?;
    let run_dir = PathBuf::from("/submissions_envs").join(submission_id.to_string());
    fs::create_dir(run_dir.clone())
        .await
        .map_log(TestingVerdict::Bug)
        .map_db(&state.db, submission_id)
        .await
        .map_http()?;
    let run_path = run_dir.join(format!(
        "run.{}",
        get_language_file_extension(&submission.language)
    ));
    let mut run_file = File::create(run_path)
        .await
        .map_log(TestingVerdict::Bug)
        .map_db(&state.db, submission_id)
        .await
        .map_http()?;
    run_file
        .write_all(&file_stream)
        .await
        .map_log(TestingVerdict::Bug)
        .map_db(&state.db, submission_id)
        .await
        .map_http()?;
    run_file
        .flush()
        .await
        .map_log(TestingVerdict::Bug)
        .map_http()?;

    let submission_task = SubmissionTask {
        problem_path: PathBuf::from("/problems").join(submission.problem_id.to_string()),
        problem_id: submission.problem_id,
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
        .map_log(TestingVerdict::Bug)
        .map_db(&state.db, submission_id)
        .await
        .map_http()?;

    Ok(Json(submission_id))
}

pub async fn retest_problem_submissions(
    State(state): State<AppState>,
    Auth(auth): Auth,
    Path(problem_id): Path<i64>,
) -> Result<(), StatusCode> {
    let problem = database::problems::get_problem(&state.db, problem_id)
        .await
        .map_http()?;
    let contest = database::contests::get_contest_by_id(&state.db, problem.contest_id)
        .await
        .map_http()?;

    if !is_allowed(auth.id, contest.owner_id, &auth.admin_level)
        && !is_allowed(auth.id, problem.owner_id, &auth.admin_level)
    {
        return Err(StatusCode::FORBIDDEN);
    }

    database::submissions::delete_subgroups_results_for_problem(&state.db, problem_id)
        .await
        .map_http()?;
    database::submissions::delete_tests_results_for_problem(&state.db, problem_id)
        .await
        .map_http()?;
    database::submissions::set_all_submissions_pending_for_problem(&state.db, problem_id)
        .await
        .map_http()?;

    for submission_id in
        database::submissions::get_user_problem_submissions(&state.db, None, problem_id)
            .await
            .map_http()?
    {
        let submission = database::submissions::get_submission(&state.db, submission_id)
            .await
            .map_http()?;
        let run_dir = PathBuf::from("/submissions_envs").join(submission_id.to_string());
        let submission_task = SubmissionTask {
            problem_path: PathBuf::from("/problems").join(submission.problem_id.to_string()),
            problem_id: submission.problem_id,
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
            .map_log(TestingVerdict::Bug)
            .map_db(&state.db, submission_id)
            .await
            .map_http()?;
    }
    Ok(())
}
