#![allow(clippy::result_large_err)]

use std::path::PathBuf;

use crate::{
    api::ApiError,
    app_state::AppState,
    checker_compiler::compile_checker,
    crypt::verify_password,
    middleware::auth::Auth,
    tools::{MapCleanupExt, is_allowed},
};
use aj_models::{
    DeletionRequest,
    contests::ContestsEvent,
    errors::{AdaJudgeError, Deletion, InvalidProblem},
    problems::{
        ProblemConfig, ProblemQuestion, ProblemQuestionRequest, ProblemsEvent, PublicProblemConfig,
    },
};
use axum::{
    Json,
    body::{Body, Bytes},
    extract::{
        Multipart, Path, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    http::header,
    response::{IntoResponse, Response},
};
use futures_util::{SinkExt, StreamExt};
use tokio::{
    fs::{self, File, read_to_string},
    io::AsyncWriteExt,
};
use tokio_util::io::ReaderStream;
use tools::map::MapHttpExt;
use uuid::Uuid;
use zip_extensions::{zip_extract::zip_extract, zip_writer::zip_create_from_directory};

pub async fn problems_ws(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Result<Response, ApiError> {
    Ok(ws.on_upgrade(move |socket| handle_problems_socket(socket, state)))
}

async fn handle_problems_socket(socket: WebSocket, state: AppState) {
    let (mut ws_tx, mut ws_rx) = socket.split();
    let problems_tx = state.problems_subs.clone();
    let mut problems_rx = problems_tx.subscribe();

    let mut send_task = tokio::spawn(async move {
        loop {
            let event = tokio::select! {
                res = problems_rx.recv() => match res {
                    Ok(e) => e,
                    Err(_) => break,
                },
                else => break,
            };
            let json = serde_json::to_string(&event).expect("serde failed");
            if ws_tx.send(Message::Text(json.into())).await.is_err() {
                break;
            }
        }
    });

    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_rx.next().await {
            if matches!(msg, Message::Close(_)) {
                break;
            }
        }
    });

    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }
}

pub async fn get_problems(
    State(state): State<AppState>,
) -> Result<Json<Vec<PublicProblemConfig>>, ApiError> {
    Ok(Json(
        database::problems::get_problems(&state.db, None)
            .await
            .map_http()?,
    ))
}

pub async fn get_my_problems(
    State(state): State<AppState>,
    Auth(auth): Auth,
) -> Result<Json<Vec<PublicProblemConfig>>, ApiError> {
    Ok(Json(
        database::problems::get_problems(&state.db, Some(auth.id))
            .await
            .map_http()?,
    ))
}

pub async fn get_problem_by_id_admin(
    State(state): State<AppState>,
    Auth(auth): Auth,
    Path(problem_id): Path<i64>,
) -> Result<Json<PublicProblemConfig>, ApiError> {
    let problem = database::problems::get_problem(&state.db, problem_id)
        .await
        .map_http()?;
    if !is_allowed(auth.id, problem.owner_id, &auth.admin_level) {
        return Err(AdaJudgeError::Forbidden).map_http()?;
    }
    Ok(Json(
        database::problems::get_problem(&state.db, problem_id)
            .await
            .map_http()?
            .into(),
    ))
}

async fn load_problem_config(
    problem_path: &std::path::Path,
) -> Result<ProblemConfig, AdaJudgeError> {
    let config_text = read_to_string(problem_path.join("config.toml"))
        .await
        .map_err(|_| AdaJudgeError::InvalidProblem(InvalidProblem::MissingConfig))?;

    toml::from_str::<ProblemConfig>(&config_text).map_err(|e| {
        AdaJudgeError::InvalidProblem(InvalidProblem::TomlError {
            message: e.message().into(),
        })
    })
}

fn validate_subgroups(config: &ProblemConfig) -> Result<(), AdaJudgeError> {
    for (i, subgroup) in config.subgroups.iter().enumerate() {
        for x in &subgroup.depends_on {
            if *x >= i {
                return Err(AdaJudgeError::InvalidProblem(
                    InvalidProblem::SubgroupConflict {
                        subgroup: i,
                        depends_on: *x,
                    },
                ));
            }
        }
        if subgroup.score.is_some() == subgroup.score_per_test.is_some() {
            return Err(AdaJudgeError::InvalidProblem(
                InvalidProblem::InvalidSubgroupScoring { subgroup: i },
            ));
        }
    }
    Ok(())
}

#[allow(clippy::too_many_lines)]
pub async fn create_problem(
    State(state): State<AppState>,
    Auth(auth): Auth,
    mut multipart: Multipart,
) -> Result<(), ApiError> {
    let mut file_stream: Option<Bytes> = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| AdaJudgeError::BadRequest)
        .map_http()?
    {
        if field.name() == Some("problem_archive") {
            file_stream = Some(
                field
                    .bytes()
                    .await
                    .map_err(|_| AdaJudgeError::BadRequest)
                    .map_http()?,
            );
        }
    }

    let Some(file_stream) = file_stream else {
        return Err(AdaJudgeError::BadRequest).map_http()?;
    };

    let request_id = Uuid::new_v4();
    let archive_path = PathBuf::from(format!("/problems/{request_id}.zip"));
    let problem_path = PathBuf::from(format!("/problems/{request_id}"));
    let mut problem_archive_file = File::create(archive_path.clone())
        .await
        .map_err(|_| AdaJudgeError::Internal)
        .map_http()?;
    problem_archive_file
        .write_all(&file_stream)
        .await
        .map_err(|_| AdaJudgeError::Internal)
        .map_http()?;
    problem_archive_file
        .flush()
        .await
        .map_err(|_| AdaJudgeError::Internal)
        .map_http()?;
    zip_extract(&archive_path, &problem_path)
        .map_err(|_| AdaJudgeError::Internal)
        .map_http()?;
    fs::remove_file(&archive_path)
        .await
        .map_err(|_| AdaJudgeError::Internal)
        .map_http()?;
    let config = match load_problem_config(&problem_path).await {
        Err(e) => {
            return Err(e).map_cleanup(&problem_path).await.map_http();
        }
        Ok(config) => config,
    };
    validate_subgroups(&config)
        .map_cleanup(&problem_path)
        .await
        .map_http()?;
    match config.owner_id {
        None => {
            return Err(AdaJudgeError::InvalidProblem(InvalidProblem::OwnerId))
                .map_cleanup(&problem_path)
                .await
                .map_http()?;
        }
        Some(owner_id) if owner_id != auth.id => {
            return Err(AdaJudgeError::InvalidProblem(InvalidProblem::OwnerId))
                .map_cleanup(&problem_path)
                .await
                .map_http()?;
        }
        _ => {}
    }
    let contest = database::contests::get_contest(&state.db, config.contest_id)
        .await
        .map_cleanup(&problem_path)
        .await
        .map_http()?;
    if !is_allowed(auth.id, contest.owner_id, &auth.admin_level)
        && contest.co_authors.binary_search(&auth.id).is_err()
    {
        return Err(AdaJudgeError::Forbidden)
            .map_cleanup(&problem_path)
            .await
            .map_http()?;
    }
    compile_checker(
        &problem_path,
        &PathBuf::from(&config.checker_path),
        &config.checker_lang,
    )
    .await
    .map_cleanup(&problem_path)
    .await
    .map_http()?;
    zip_create_from_directory(&archive_path, &problem_path)
        .map_err(|_| AdaJudgeError::Internal)
        .map_cleanup(&problem_path)
        .await
        .map_http()?;
    let problem_id = database::problems::create_problem(&state.db, auth.id, &config)
        .await
        .map_cleanup(&problem_path)
        .await
        .map_http()?;
    let new_problem_path = PathBuf::from(format!("/problems/{problem_id}"));
    let new_problem_archive_path = PathBuf::from(format!("/problems/{problem_id}.zip"));

    fs::rename(&problem_path, &new_problem_path)
        .await
        .map_err(|_| AdaJudgeError::Internal)
        .map_http()?;
    fs::rename(&archive_path, &new_problem_archive_path)
        .await
        .map_err(|_| AdaJudgeError::Internal)
        .map_http()?;
    state
        .contests_subs
        .get(&Some(contest.id))
        .map(|tx| tx.send(ContestsEvent::NewProblem(problem_id)));
    state
        .problems_subs
        .send(ProblemsEvent::NewProblem(problem_id))
        .map_err(|_| AdaJudgeError::Internal)
        .map_http()?;

    Ok(())
}

#[allow(clippy::too_many_lines)]
pub async fn update_problem(
    State(state): State<AppState>,
    Auth(auth): Auth,
    Path(problem_id): Path<i64>,
    mut multipart: Multipart,
) -> Result<(), ApiError> {
    let mut file_stream: Option<Bytes> = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| AdaJudgeError::BadRequest)
        .map_http()?
    {
        if field.name() == Some("problem_archive") {
            file_stream = Some(
                field
                    .bytes()
                    .await
                    .map_err(|_| AdaJudgeError::BadRequest)
                    .map_http()?,
            );
        }
    }

    let Some(file_stream) = file_stream else {
        return Err(AdaJudgeError::BadRequest).map_http()?;
    };

    let request_id = Uuid::new_v4();
    let archive_path = PathBuf::from(format!("/problems/{request_id}.zip"));
    let problem_path = PathBuf::from(format!("/problems/{request_id}"));
    let mut problem_archive_file = File::create(archive_path.clone())
        .await
        .map_err(|_| AdaJudgeError::Internal)
        .map_http()?;
    problem_archive_file
        .write_all(&file_stream)
        .await
        .map_err(|_| AdaJudgeError::Internal)
        .map_http()?;
    problem_archive_file
        .flush()
        .await
        .map_err(|_| AdaJudgeError::Internal)
        .map_http()?;
    zip_extract(&archive_path, &problem_path)
        .map_err(|_| AdaJudgeError::Internal)
        .map_http()?;
    fs::remove_file(&archive_path)
        .await
        .map_err(|_| AdaJudgeError::Internal)
        .map_http()?;
    let config = match load_problem_config(&problem_path).await {
        Err(e) => {
            return Err(e).map_cleanup(&problem_path).await.map_http();
        }
        Ok(config) => config,
    };
    validate_subgroups(&config)
        .map_cleanup(&problem_path)
        .await
        .map_http()?;
    match config.owner_id {
        None => {
            return Err(AdaJudgeError::InvalidProblem(InvalidProblem::OwnerId))
                .map_cleanup(&problem_path)
                .await
                .map_http()?;
        }
        Some(owner_id) if owner_id != auth.id => {
            return Err(AdaJudgeError::InvalidProblem(InvalidProblem::OwnerId))
                .map_cleanup(&problem_path)
                .await
                .map_http()?;
        }
        _ => {}
    }
    let contest = database::contests::get_contest(&state.db, config.contest_id)
        .await
        .map_cleanup(&problem_path)
        .await
        .map_http()?;
    if !is_allowed(auth.id, contest.owner_id, &auth.admin_level)
        && !is_allowed(auth.id, config.owner_id, &auth.admin_level)
    {
        return Err(AdaJudgeError::Forbidden)
            .map_cleanup(&problem_path)
            .await
            .map_http()?;
    }
    compile_checker(
        &problem_path,
        &PathBuf::from(&config.checker_path),
        &config.checker_lang,
    )
    .await
    .map_cleanup(&problem_path)
    .await
    .map_http()?;
    zip_create_from_directory(&archive_path, &problem_path)
        .map_err(|_| AdaJudgeError::Internal)
        .map_cleanup(&problem_path)
        .await
        .map_http()?;
    let new_problem_path = PathBuf::from(format!("/problems/{problem_id}"));
    let new_problem_archive_path = PathBuf::from(format!("/problems/{problem_id}.zip"));
    fs::remove_dir_all(&new_problem_path)
        .await
        .map_err(|_| AdaJudgeError::Internal)
        .map_http()?;
    fs::remove_file(&new_problem_archive_path)
        .await
        .map_err(|_| AdaJudgeError::Internal)
        .map_http()?;
    fs::rename(&problem_path, &new_problem_path)
        .await
        .map_err(|_| AdaJudgeError::Internal)
        .map_http()?;
    fs::rename(&archive_path, &new_problem_archive_path)
        .await
        .map_err(|_| AdaJudgeError::Internal)
        .map_http()?;
    database::problems::update_problem(&state.db, problem_id, &config)
        .await
        .map_cleanup(&new_problem_path)
        .await
        .map_http()?;
    state
        .contests_subs
        .get(&Some(contest.id))
        .map(|tx| tx.send(ContestsEvent::ProblemUpdated(problem_id)));
    state
        .problems_subs
        .send(ProblemsEvent::ProblemUpdated(problem_id))
        .map_err(|_| AdaJudgeError::Internal)
        .map_http()?;

    Ok(())
}

#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
pub async fn delete_problem(
    State(state): State<AppState>,
    Path(problem_id): Path<i64>,
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
        let problem = database::problems::get_problem(&state.db, problem_id)
            .await
            .map_http()?;
        let contest = database::contests::get_contest(&state.db, problem.contest_id)
            .await
            .map_http()?;
        if is_allowed(auth.id, problem.owner_id, &auth.admin_level)
            || is_allowed(auth.id, contest.owner_id, &auth.admin_level)
        {
            let submissions =
                database::submissions::get_problem_submissions(&state.db, None, problem_id)
                    .await
                    .map_http()?;
            for submission in submissions {
                let submission_id = submission.id;
                fs::remove_dir_all(PathBuf::from(format!("/submissions_envs/{submission_id}")))
                    .await
                    .map_err(|_| AdaJudgeError::Internal)
                    .map_http()?;
            }

            database::problems::delete_problem(&state.db, problem_id)
                .await
                .map_http()?;
            fs::remove_dir_all(PathBuf::from(format!("/problems/{problem_id}")))
                .await
                .map_err(|_| AdaJudgeError::Internal)
                .map_http()?;
            fs::remove_file(PathBuf::from(format!("/problems/{problem_id}.zip")))
                .await
                .map_err(|_| AdaJudgeError::Internal)
                .map_http()?;
            state
                .contests_subs
                .get(&Some(contest.id))
                .map(|tx| tx.send(ContestsEvent::ProblemDeleted(problem_id)));
            state
                .problems_subs
                .send(ProblemsEvent::ProblemDeleted(problem_id))
                .map_err(|_| AdaJudgeError::Internal)
                .map_http()?;
            Ok(())
        } else {
            Err(AdaJudgeError::Forbidden).map_http()?
        }
    } else {
        Err(AdaJudgeError::Deletion(Deletion::InvalidLoginOrPassword)).map_http()?
    }
}

pub async fn create_problem_question(
    State(state): State<AppState>,
    Path(problem_id): Path<i64>,
    Auth(auth): Auth,
    Json(request): Json<ProblemQuestionRequest>,
) -> Result<(), ApiError> {
    let id = database::problems::create_problem_question(&state.db, auth.id, problem_id, &request)
        .await
        .map_http()?;
    let question = database::problems::get_problem_question(&state.db, id)
        .await
        .map_http()?;
    let problem = database::problems::get_problem(&state.db, question.problem_id)
        .await
        .map_http()?;
    state
        .contests_subs
        .get(&Some(problem.contest_id))
        .map(|tx| tx.send(ContestsEvent::NewProblemQuestion(id)));
    Ok(())
}

pub async fn answer_problem_question(
    State(state): State<AppState>,
    Path(question_id): Path<i64>,
    Auth(auth): Auth,
    Json(request): Json<String>,
) -> Result<(), ApiError> {
    let question = database::problems::get_problem_question(&state.db, question_id)
        .await
        .map_http()?;
    let problem = database::problems::get_problem(&state.db, question.problem_id)
        .await
        .map_http()?;
    let contest = database::contests::get_contest(&state.db, problem.contest_id)
        .await
        .map_http()?;
    if !is_allowed(auth.id, problem.owner_id, &auth.admin_level)
        && !is_allowed(auth.id, contest.owner_id, &auth.admin_level)
    {
        return Err(AdaJudgeError::Forbidden).map_http()?;
    }
    database::problems::answer_problem_question(&state.db, question_id, &request)
        .await
        .map_http()?;
    state
        .contests_subs
        .get(&Some(problem.contest_id))
        .map(|tx| tx.send(ContestsEvent::ProblemQuestionAnswered(question_id)));

    Ok(())
}

#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
pub async fn delete_problem_question(
    State(state): State<AppState>,
    Path(question_id): Path<i64>,
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
        let question = database::problems::get_problem_question(&state.db, question_id)
            .await
            .map_http()?;
        let problem = database::problems::get_problem(&state.db, question.problem_id)
            .await
            .map_http()?;
        let contest = database::contests::get_contest(&state.db, problem.contest_id)
            .await
            .map_http()?;
        if is_allowed(auth.id, Some(question.owner_id), &auth.admin_level)
            || is_allowed(auth.id, problem.owner_id, &auth.admin_level)
            || is_allowed(auth.id, contest.owner_id, &auth.admin_level)
        {
            database::problems::delete_problem_question(&state.db, question_id)
                .await
                .map_http()?;
            state
                .contests_subs
                .get(&Some(problem.contest_id))
                .map(|tx| tx.send(ContestsEvent::ProblemQuestionDeleted(question_id)));
            Ok(())
        } else {
            Err(AdaJudgeError::Forbidden).map_http()?
        }
    } else {
        Err(AdaJudgeError::Deletion(Deletion::InvalidLoginOrPassword)).map_http()?
    }
}

pub async fn get_problem_question_by_id(
    State(state): State<AppState>,
    Auth(auth): Auth,
    Path(question_id): Path<i64>,
) -> Result<Json<ProblemQuestion>, ApiError> {
    let question = database::problems::get_problem_question(&state.db, question_id)
        .await
        .map_http()?;
    let problem = database::problems::get_problem(&state.db, question.problem_id)
        .await
        .map_http()?;
    let contest = database::contests::get_contest(&state.db, problem.contest_id)
        .await
        .map_http()?;
    if !is_allowed(auth.id, problem.owner_id, &auth.admin_level)
        && !is_allowed(auth.id, contest.owner_id, &auth.admin_level)
        && !is_allowed(auth.id, Some(question.owner_id), &auth.admin_level)
    {
        return Err(AdaJudgeError::Forbidden).map_http()?;
    }

    Ok(Json(question))
}

pub async fn download_problem(
    State(state): State<AppState>,
    Path(problem_id): Path<i64>,
    Auth(auth): Auth,
) -> Result<impl IntoResponse, ApiError> {
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
        Err(AdaJudgeError::Forbidden).map_http()?
    } else {
        let file_path = PathBuf::from(format!("/problems/{problem_id}.zip"));
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
                format!("attachment; filename=\"{problem_id}.zip\""),
            ),
        ];

        Ok((headers, body))
    }
}
