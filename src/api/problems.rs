use std::path::PathBuf;

use crate::{
    app_state::AppState, crypt::verify_password, middleware::auth::Auth, tools::is_allowed,
};
use aj_models::{
    DeletionRequest,
    contests::PublicContestConfig,
    errors::{AdaJudgeError, InvalidProblem},
    problems::{ProblemConfig, ProblemQuestion, ProblemQuestionRequest, PublicProblemConfig},
    verdicts::TestingVerdict,
};
use axum::{
    Json,
    body::{Body, Bytes},
    extract::{Multipart, Path, State},
    http::{StatusCode, header},
    response::IntoResponse,
};
use database::problems::get_problems;
use tokio::{
    fs::{self, File, read_to_string},
    io::AsyncWriteExt,
};
use tokio_util::io::ReaderStream;
use tools::map::{MapHttpExt, MapLogExt};
use uuid::Uuid;
use zip_extensions::zip_extract::zip_extract;

pub async fn get_problems(State(state): State<AppState>) -> Result<Json<Vec<i64>>, StatusCode> {
    Ok(Json(get_problems(&state.db, None).await.map_http()?))
}

pub async fn get_my_problems(
    State(state): State<AppState>,
    Auth(auth): Auth,
) -> Result<Json<Vec<i64>>, StatusCode> {
    Ok(Json(
        get_problems(&state.db, Some(auth.id)).await.map_http()?,
    ))
}

pub async fn get_problem_by_id_admin(
    State(state): State<AppState>,
    Auth(auth): Auth,
    Path(problem_id): Path<i64>,
) -> Result<Json<PublicProblemConfig>, StatusCode> {
    let problem = database::problems::get_problem(&state.db, problem_id)
        .await
        .map_http()?;
    if !is_allowed(auth.id, problem.owner_id, &auth.admin_level) {
        return Err(StatusCode::FORBIDDEN);
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
) -> Result<ProblemConfig, TestingVerdict> {
    let config_text = read_to_string(problem_path.join("config.toml"))
        .await
        .map_log(TestingVerdict::InvalidProblem)?;

    toml::from_str::<ProblemConfig>(&config_text).map_log(TestingVerdict::InvalidProblem)
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

pub async fn create_problem(
    State(state): State<AppState>,
    Auth(auth): Auth,
    mut multipart: Multipart,
) -> Result<Json<i64>, StatusCode> {
    let mut file_stream: Option<Bytes> = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_log(TestingVerdict::InvalidRequest)
        .map_http()?
    {
        match field.name() {
            Some("problem_archive") => {
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

    let Some(file_stream) = file_stream else {
        return Err(StatusCode::BAD_REQUEST);
    };

    let request_id = Uuid::new_v4();
    let archive_path = PathBuf::from(format!("/problems/{request_id}.zip"));
    let problem_path = PathBuf::from(format!("/problems/{request_id}"));
    let mut problem_archive_file = File::create(archive_path.clone())
        .await
        .map_log(TestingVerdict::Bug)
        .map_http()?;
    problem_archive_file
        .write_all(&file_stream)
        .await
        .map_log(TestingVerdict::Bug)
        .map_http()?;
    problem_archive_file
        .flush()
        .await
        .map_log(TestingVerdict::Bug)
        .map_http()?;
    zip_extract(&archive_path, &problem_path)
        .map_log(TestingVerdict::Bug)
        .map_http()?;
    let config = match load_problem_config(&problem_path).await {
        Err(e) => {
            std::fs::remove_dir_all(&problem_path)
                .map_log(TestingVerdict::Bug)
                .map_http()?;
            return Err(e).map_http();
        }
        Ok(config) => config,
    };
    validate_subgroups(&config).map_http()?;
    match config.owner_id {
        None => return Err(StatusCode::BAD_REQUEST),
        Some(owner_id) if owner_id != auth.id => return Err(StatusCode::FORBIDDEN),
        _ => {}
    }
    let contest = database::contests::get_contest_by_id(&state.db, config.contest_id)
        .await
        .map_http()?;
    if !is_allowed(auth.id, contest.owner_id, &auth.admin_level)
        && contest.co_authors.binary_search(&auth.id).is_err()
    {
        return Err(StatusCode::FORBIDDEN);
    }
    let problem_id = database::problems::create_problem(
        &state.db,
        auth.id,
        config.r#type,
        config.merge_subgroups,
        config.contest_id,
        config.problem_index,
        &config.name_ru,
        &config.name_en,
        config.time_limit_ms,
        config.memory_limit_mb,
        &config.checker_path,
        &config.tests_path,
    )
    .await
    .map_http()?;
    let new_problem_path = PathBuf::from(format!("/problems/{}", problem_id));
    let new_problem_archive_path = PathBuf::from(format!("/problems/{}.zip", problem_id));

    fs::rename(problem_path, new_problem_path)
        .await
        .map_log(TestingVerdict::Bug)
        .map_http()?;
    fs::rename(archive_path, new_problem_archive_path)
        .await
        .map_log(TestingVerdict::Bug)
        .map_http()?;

    database::problems::insert_problem_subgroups(&state.db, problem_id, &config.subgroups)
        .await
        .map_http()?;

    Ok(Json(problem_id))
}

pub async fn update_problem(
    State(state): State<AppState>,
    Auth(auth): Auth,
    Path(problem_id): Path<i64>,
    mut multipart: Multipart,
) -> Result<(), StatusCode> {
    let mut file_stream: Option<Bytes> = None;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_log(TestingVerdict::InvalidRequest)
        .map_http()?
    {
        match field.name() {
            Some("problem_archive") => {
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

    let Some(file_stream) = file_stream else {
        return Err(StatusCode::BAD_REQUEST);
    };

    let request_id = Uuid::new_v4();
    let archive_path = PathBuf::from(format!("/problems/{request_id}.zip"));
    let problem_path = PathBuf::from(format!("/problems/{request_id}"));
    let mut problem_archive_file = File::create(archive_path.clone())
        .await
        .map_log(TestingVerdict::Bug)
        .map_http()?;
    problem_archive_file
        .write_all(&file_stream)
        .await
        .map_log(TestingVerdict::Bug)
        .map_http()?;
    problem_archive_file
        .flush()
        .await
        .map_log(TestingVerdict::Bug)
        .map_http()?;
    zip_extract(&archive_path, &problem_path)
        .map_log(TestingVerdict::Bug)
        .map_http()?;
    let config = match load_problem_config(&problem_path).await {
        Err(e) => {
            std::fs::remove_dir_all(&problem_path)
                .map_log(TestingVerdict::Bug)
                .map_http()?;
            return Err(e).map_http();
        }
        Ok(config) => config,
    };
    validate_subgroups(&config).map_http()?;
    match config.owner_id {
        None => return Err(StatusCode::BAD_REQUEST),
        Some(owner_id) if owner_id != auth.id => return Err(StatusCode::FORBIDDEN),
        _ => {}
    }
    let problem = database::problems::get_problem(&state.db, problem_id)
        .await
        .map_http()?;
    let contest = database::contests::get_contest_by_id(&state.db, config.contest_id)
        .await
        .map_http()?;
    if !is_allowed(auth.id, contest.owner_id, &auth.admin_level)
        && !is_allowed(auth.id, problem.owner_id, &auth.admin_level)
    {
        return Err(StatusCode::FORBIDDEN);
    }
    let new_problem_path = PathBuf::from(format!("/problems/{}", problem_id));
    let new_problem_archive_path = PathBuf::from(format!("/problems/{}.zip", problem_id));

    fs::remove_dir_all(&new_problem_path)
        .await
        .map_log(TestingVerdict::Bug)
        .map_http()?;
    fs::rename(problem_path, new_problem_path)
        .await
        .map_log(TestingVerdict::Bug)
        .map_http()?;
    fs::rename(archive_path, new_problem_archive_path)
        .await
        .map_log(TestingVerdict::Bug)
        .map_http()?;
    database::problems::update_problem(
        &state.db,
        problem_id,
        config.r#type,
        config.merge_subgroups,
        config.contest_id,
        config.problem_index,
        &config.name_ru,
        &config.name_en,
        config.time_limit_ms,
        config.memory_limit_mb,
        &config.checker_path,
        &config.tests_path,
    )
    .await
    .map_http()?;

    database::problems::insert_problem_subgroups(&state.db, problem_id, &config.subgroups)
        .await
        .map_http()?;

    Ok(())
}

pub async fn delete_problem(
    State(state): State<AppState>,
    Path(problem_id): Path<i64>,
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
        let problem = database::problems::get_problem(&state.db, problem_id)
            .await
            .map_http()?;
        if !is_allowed(auth.id, problem.owner_id, &auth.admin_level) {
            Err(StatusCode::FORBIDDEN)
        } else {
            database::problems::delete_problem(&state.db, problem_id)
                .await
                .map_http()?;
            fs::remove_dir_all(PathBuf::from(format!("/problems/{problem_id}")))
                .await
                .map_log(TestingVerdict::Bug)
                .map_http()?;
            Ok(())
        }
    }
}

pub async fn create_problem_question(
    State(state): State<AppState>,
    Path(problem_id): Path<i64>,
    Auth(auth): Auth,
    Json(request): Json<ProblemQuestionRequest>,
) -> Result<Json<i64>, StatusCode> {
    Ok(Json(
        database::problems::create_problem_question(
            &state.db,
            auth.id,
            problem_id,
            &request.title,
            &request.text,
        )
        .await
        .map_http()?,
    ))
}

pub async fn answer_problem_question(
    State(state): State<AppState>,
    Path(question_id): Path<i64>,
    Auth(auth): Auth,
    Json(request): Json<String>,
) -> Result<(), StatusCode> {
    let question = database::problems::get_problem_question(&state.db, question_id)
        .await
        .map_http()?;
    let problem = database::problems::get_problem(&state.db, question.problem_id)
        .await
        .map_http()?;
    let contest = database::contests::get_contest_by_id(&state.db, problem.contest_id)
        .await
        .map_http()?;
    if !is_allowed(auth.id, problem.owner_id, &auth.admin_level)
        && !is_allowed(auth.id, contest.owner_id, &auth.admin_level)
    {
        return Err(StatusCode::FORBIDDEN);
    }
    database::problems::answer_problem_question(&state.db, question_id, &request)
        .await
        .map_http()?;

    Ok(())
}

pub async fn delete_problem_question(
    State(state): State<AppState>,
    Path(question_id): Path<i64>,
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
        let question = database::problems::get_problem_question(&state.db, question_id)
            .await
            .map_http()?;
        if !is_allowed(auth.id, Some(question.owner_id), &auth.admin_level) {
            Err(StatusCode::FORBIDDEN)
        } else {
            database::problems::delete_problem_question(&state.db, question_id)
                .await
                .map_http()?;
            Ok(())
        }
    }
}

pub async fn get_problem_question_by_id(
    State(state): State<AppState>,
    Auth(auth): Auth,
    Path(question_id): Path<i64>,
) -> Result<Json<ProblemQuestion>, StatusCode> {
    let question = database::problems::get_problem_question(&state.db, question_id)
        .await
        .map_http()?;
    let problem = database::problems::get_problem(&state.db, question.problem_id)
        .await
        .map_http()?;
    let contest = database::contests::get_contest_by_id(&state.db, problem.contest_id)
        .await
        .map_http()?;
    if !is_allowed(auth.id, problem.owner_id, &auth.admin_level)
        && !is_allowed(auth.id, contest.owner_id, &auth.admin_level)
        && !is_allowed(auth.id, Some(question.owner_id), &auth.admin_level)
    {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(Json(question))
}

pub async fn get_all_problem_questions(
    State(state): State<AppState>,
    Auth(auth): Auth,
    Path(problem_id): Path<i64>,
) -> Result<Json<Vec<i64>>, StatusCode> {
    let problem = database::problems::get_problem(&state.db, problem_id)
        .await
        .map_http()?;
    let contest = database::contests::get_contest_by_id(&state.db, problem.contest_id)
        .await
        .map_http()?;
    if !is_allowed(auth.id, problem.owner_id, &auth.admin_level)
        && !is_allowed(auth.id, contest.owner_id, &auth.admin_level)
    {
        return Err(StatusCode::FORBIDDEN);
    }

    Ok(Json(
        database::problems::get_problem_questions(&state.db, None, problem_id)
            .await
            .map_http()?,
    ))
}

pub async fn get_my_problem_questions(
    State(state): State<AppState>,
    Auth(auth): Auth,
    Path(problem_id): Path<i64>,
) -> Result<Json<Vec<i64>>, StatusCode> {
    Ok(Json(
        database::problems::get_problem_questions(&state.db, Some(auth.id), problem_id)
            .await
            .map_http()?,
    ))
}

pub async fn download_problem(
    State(state): State<AppState>,
    Path(problem_id): Path<i64>,
    Auth(auth): Auth,
) -> Result<impl IntoResponse, StatusCode> {
    let problem = database::problems::get_problem(&state.db, problem_id)
        .await
        .map_http()?;
    let contest = database::contests::get_contest_by_id(&state.db, problem.contest_id)
        .await
        .map_http()?;

    if !is_allowed(auth.id, problem.owner_id, &auth.admin_level)
        && !is_allowed(auth.id, contest.owner_id, &auth.admin_level)
        && contest.co_authors.binary_search(&auth.id).is_err()
    {
        Err(StatusCode::FORBIDDEN)
    } else {
        let file_path = PathBuf::from(format!("/problems/{problem_id}.zip"));
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
                format!("attachment; filename=\"{problem_id}.zip\""),
            ),
        ];

        Ok((headers, body))
    }
}
