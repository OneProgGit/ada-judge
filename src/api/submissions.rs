use crate::middleware::contests::check_contest_started_and_not_ended_common;
use crate::{app_state::AppState, middleware::auth::Auth};
use ada_judge_public_models::testing::get_language_file_extension;
use ada_judge_public_models::users::AdminLevel;
use ada_judge_public_models::{
    testing::{Submission, SubmissonRequest},
    verdicts::TotalVerdict,
};
use apalis::prelude::TaskSink;
use axum::{
    Json,
    body::Bytes,
    extract::{Multipart, Path, State},
    http::StatusCode,
};
use database::tools::MapDbExt;
use models::testing::SubmissionTask;
use std::{path::PathBuf, sync::Arc};
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};
use tools::map::{MapHttpExt, MapLogExt};

pub async fn get_all_my_submissions(
    State(state): State<Arc<AppState>>,
    Auth(auth): Auth,
) -> Result<Json<Vec<i64>>, StatusCode> {
    log::info!("Get all my submissions");
    Ok(Json(
        database::submissions::get_all_user_submissions(&state.db, auth.id)
            .await
            .map_http()?,
    ))
}

pub async fn get_contest_my_submissions(
    State(state): State<Arc<AppState>>,
    Path(contest_id): Path<i64>,
    Auth(auth): Auth,
) -> Result<Json<Vec<i64>>, StatusCode> {
    log::info!("Get contest my submissions");
    Ok(Json(
        database::submissions::get_contest_user_submissions(&state.db, auth.id, contest_id)
            .await
            .map_http()?,
    ))
}

pub async fn get_problem_my_submissions(
    State(state): State<Arc<AppState>>,
    Path(problem_id): Path<i64>,
    Auth(auth): Auth,
) -> Result<Json<Vec<i64>>, StatusCode> {
    log::info!("Get problem my submissions");
    Ok(Json(
        database::submissions::get_problem_user_submissions(&state.db, auth.id, problem_id)
            .await
            .map_http()?,
    ))
}

pub async fn get_all_user_submissions(
    State(state): State<Arc<AppState>>,
    Path(user_id): Path<i64>,
) -> Result<Json<Vec<i64>>, StatusCode> {
    log::info!("Get all user submissions");
    Ok(Json(
        database::submissions::get_all_user_submissions(&state.db, user_id)
            .await
            .map_http()?,
    ))
}

pub async fn get_contest_user_submissions(
    State(state): State<Arc<AppState>>,
    Path((contest_id, user_id)): Path<(i64, i64)>,
) -> Result<Json<Vec<i64>>, StatusCode> {
    log::info!("Get contest user submissions");
    Ok(Json(
        database::submissions::get_contest_user_submissions(&state.db, user_id, contest_id)
            .await
            .map_http()?,
    ))
}

pub async fn get_problem_user_submissions(
    State(state): State<Arc<AppState>>,
    Path((problem_id, user_id)): Path<(i64, i64)>,
) -> Result<Json<Vec<i64>>, StatusCode> {
    log::info!("Get problem user submissions");
    Ok(Json(
        database::submissions::get_problem_user_submissions(&state.db, user_id, problem_id)
            .await
            .map_http()?,
    ))
}

pub async fn get_all_submissions(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<i64>>, StatusCode> {
    log::info!("Get all submissions");
    Ok(Json(
        database::submissions::get_all_user_submissions(&state.db, -1)
            .await
            .map_http()?,
    ))
}

pub async fn get_contest_submissions(
    State(state): State<Arc<AppState>>,
    Path(contest_id): Path<i64>,
) -> Result<Json<Vec<i64>>, StatusCode> {
    log::info!("Get contest submissions");
    Ok(Json(
        database::submissions::get_contest_user_submissions(&state.db, -1, contest_id)
            .await
            .map_http()?,
    ))
}

pub async fn get_problem_submissions(
    State(state): State<Arc<AppState>>,
    Path(problem_id): Path<i64>,
) -> Result<Json<Vec<i64>>, StatusCode> {
    log::info!("Get problem submissions");
    Ok(Json(
        database::submissions::get_problem_user_submissions(&state.db, -1, problem_id)
            .await
            .map_http()?,
    ))
}

pub async fn get_submission(
    State(state): State<Arc<AppState>>,
    Path(submission_id): Path<i64>,
    Auth(auth): Auth,
) -> Result<Json<Submission>, StatusCode> {
    log::info!("Get problem user submissions");

    let submission: Submission = database::submissions::get_submission(&state.db, submission_id)
        .await
        .map_http()?
        .into();

    if submission.user_id != auth.id && auth.admin_level != AdminLevel::Owner {
        Err(StatusCode::BAD_REQUEST)
    } else {
        Ok(Json(submission))
    }
}

pub async fn push_submission_to_queue(
    State(state): State<Arc<AppState>>,
    Auth(auth): Auth,
    mut multipart: Multipart,
) -> Result<Json<i64>, StatusCode> {
    let mut submission: Option<SubmissonRequest> = None;
    let mut file_stream: Option<Bytes> = None;

    log::info!("Extracting submission data and file");
    while let Some(field) = multipart
        .next_field()
        .await
        .map_log(TotalVerdict::InvalidRequest)
        .map_http()?
    {
        match field.name() {
            Some("submission_data") => {
                let text = field
                    .text()
                    .await
                    .map_log(TotalVerdict::InvalidRequest)
                    .map_http()?;
                submission = Some(
                    serde_json::from_str(&text)
                        .map_log(TotalVerdict::InvalidRequest)
                        .map_http()?,
                );
            }
            Some("submission_file") => {
                file_stream = Some(field.bytes().await.map_log(TotalVerdict::Bug).map_http()?);
            }
            _ => {}
        }
    }

    let Some(submission) = submission else {
        log::error!("No submission data was provided");
        return Err(StatusCode::BAD_REQUEST);
    };

    let Some(file_stream) = file_stream else {
        log::error!("No submission files were provided");
        return Err(StatusCode::BAD_REQUEST);
    };

    let problem = database::get_problem_by_id(&state.db, submission.problem_id)
        .await
        .map_http()?;

    if let Err(e) = check_contest_started_and_not_ended_common(
        &state.db,
        auth.id,
        problem.contest_id,
        auth.admin_level,
    )
    .await
    {
        return Err(e.status());
    }

    log::info!("Push to queue: {submission:?}");

    let submission_id = database::submissions::insert_submission(
        &state.db,
        auth.id,
        submission.problem_id,
        &submission.language,
    )
    .await
    .map_http()?;

    log::info!("Create env dir");
    let run_dir = PathBuf::from("/submissions_envs").join(submission_id.to_string());
    fs::create_dir(run_dir.clone())
        .await
        .map_log(TotalVerdict::Bug)
        .map_db(&state.db, submission_id)
        .await
        .map_http()?;

    log::info!("Create submission file");
    let run_path = run_dir.join(format!(
        "run.{}",
        get_language_file_extension(&submission.language)
    ));
    let mut run_file = File::create(run_path)
        .await
        .map_log(TotalVerdict::Bug)
        .map_db(&state.db, submission_id)
        .await
        .map_http()?;
    run_file
        .write_all(&file_stream)
        .await
        .map_log(TotalVerdict::Bug)
        .map_db(&state.db, submission_id)
        .await
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
        .map_log(TotalVerdict::Bug)
        .map_db(&state.db, submission_id)
        .await
        .map_http()?;

    Ok(Json(submission_id))
}
