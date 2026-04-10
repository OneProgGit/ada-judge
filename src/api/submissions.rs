use crate::{app_state::AppState, middleware::auth::Auth};
use apalis::prelude::TaskSink;
use axum::{
    Json,
    body::Bytes,
    extract::{Multipart, Path, State},
    http::StatusCode,
};
use database::tools::MapDbExt;
use models::{
    testing::{Submission, SubmissionTask, SubmissonRequest, get_lang_str},
    verdicts::TotalVerdict,
};
use std::{path::PathBuf, sync::Arc};
use tokio::{
    fs::{self, File},
    io::AsyncWriteExt,
};
use tools::map::{MapHttpExt, MapLogExt};

pub async fn get_all_user_submisssions(
    State(state): State<Arc<AppState>>,
    Auth(auth): Auth,
) -> Result<Json<Vec<Submission>>, StatusCode> {
    log::info!("Get all user submissions");
    Ok(Json(
        database::submissions::get_all_user_submissions(&state.db, auth.id)
            .await
            .map_http()?
            .iter()
            .map(Into::into)
            .collect(),
    ))
}

pub async fn get_contest_user_submissions(
    State(state): State<Arc<AppState>>,
    Path(contest_id): Path<i64>,
    Auth(auth): Auth,
) -> Result<Json<Vec<Submission>>, StatusCode> {
    log::info!("Get contest user submissions");
    Ok(Json(
        database::submissions::get_contest_user_submissions(&state.db, auth.id, contest_id)
            .await
            .map_http()?
            .iter()
            .map(Into::into)
            .collect(),
    ))
}

pub async fn get_problem_user_submissions(
    State(state): State<Arc<AppState>>,
    Path(problem_id): Path<i64>,
    Auth(auth): Auth,
) -> Result<Json<Vec<Submission>>, StatusCode> {
    log::info!("Get problem user submissions");
    Ok(Json(
        database::submissions::get_problem_user_submissions(&state.db, auth.id, problem_id)
            .await
            .map_http()?
            .iter()
            .map(Into::into)
            .collect(),
    ))
}

pub async fn push_submission_to_queue(
    State(state): State<Arc<AppState>>,
    Auth(user): Auth,
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

    log::info!("Push to queue: {submission:?}");

    let submission_id =
        database::submissions::insert_submission(&state.db, user.id, submission.problem_id)
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
    let run_path = run_dir.join(format!("run.{}", get_lang_str(&submission.lang)));
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
        lang: submission.lang,
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
