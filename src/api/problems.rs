use std::path::PathBuf;

use crate::{
    app_state::AppState, crypt::verify_password, middleware::auth::Auth, tools::is_allowed,
};
use ada_judge_public_models::{
    DeletionRequest,
    problems::{ProblemConfig, PublicProblemConfig},
    verdicts::TotalVerdict,
};
use axum::{
    Json,
    body::Bytes,
    extract::{Multipart, Path, State},
    http::StatusCode,
};
use database::problems::get_all_user_problems;
use tokio::{
    fs::{self, File, read_to_string},
    io::AsyncWriteExt,
};
use tools::map::{MapHttpExt, MapLogExt};
use uuid::Uuid;
use zip_extensions::zip_extract::zip_extract;

pub async fn get_problems(State(state): State<AppState>) -> Result<Json<Vec<i64>>, StatusCode> {
    Ok(Json(
        get_all_user_problems(&state.db, None).await.map_http()?,
    ))
}

pub async fn get_my_problems(
    State(state): State<AppState>,
    Auth(auth): Auth,
) -> Result<Json<Vec<i64>>, StatusCode> {
    Ok(Json(
        get_all_user_problems(&state.db, Some(auth.id))
            .await
            .map_http()?,
    ))
}

pub async fn get_problem_by_id_admin(
    State(state): State<AppState>,
    Auth(auth): Auth,
    Path(problem_id): Path<i64>,
) -> Result<Json<PublicProblemConfig>, StatusCode> {
    let problem = database::problems::get_problem_by_id(&state.db, problem_id)
        .await
        .map_http()?;
    if !is_allowed(auth.id, problem.owner_id, &auth.admin_level) {
        return Err(StatusCode::FORBIDDEN);
    }
    Ok(Json(
        database::problems::get_problem_by_id(&state.db, problem_id)
            .await
            .map_http()?
            .into(),
    ))
}

async fn load_problem_config(
    problem_path: &std::path::Path,
) -> Result<ProblemConfig, TotalVerdict> {
    let config_text = read_to_string(problem_path.join("config.toml"))
        .await
        .map_log(TotalVerdict::InvalidProblem)?;

    toml::from_str::<ProblemConfig>(&config_text).map_log(TotalVerdict::InvalidProblem)
}

pub async fn create_problem(
    State(state): State<AppState>,
    Auth(auth): Auth,
    mut multipart: Multipart,
) -> Result<Json<i64>, StatusCode> {
    let mut file_stream: Option<Bytes> = None;

    log::info!("Extracting file");
    while let Some(field) = multipart
        .next_field()
        .await
        .map_log(TotalVerdict::InvalidRequest)
        .map_http()?
    {
        match field.name() {
            Some("problem_archive") => {
                file_stream = Some(field.bytes().await.map_log(TotalVerdict::Bug).map_http()?);
            }
            _ => {}
        }
    }

    let Some(file_stream) = file_stream else {
        log::error!("No problem files were provided");
        return Err(StatusCode::BAD_REQUEST);
    };

    let request_id = Uuid::new_v4();
    let archive_path = PathBuf::from(format!("/problems/{request_id}.zip"));
    let problem_path = PathBuf::from(format!("/problems/{request_id}"));

    log::info!("Create problem file");

    let mut problem_archive_file = File::create(archive_path.clone())
        .await
        .map_log(TotalVerdict::Bug)
        .map_http()?;
    problem_archive_file
        .write_all(&file_stream)
        .await
        .map_log(TotalVerdict::Bug)
        .map_http()?;
    problem_archive_file
        .flush()
        .await
        .map_log(TotalVerdict::Bug)
        .map_http()?;

    log::info!("Extract problem archive");

    zip_extract(&archive_path, &problem_path)
        .map_log(TotalVerdict::Bug)
        .map_http()?;
    fs::remove_file(archive_path)
        .await
        .map_log(TotalVerdict::Bug)
        .map_http()?;

    log::info!("Insert problem to database");
    let config = match load_problem_config(&problem_path).await {
        Err(e) => {
            std::fs::remove_dir_all(&problem_path)
                .map_log(TotalVerdict::Bug)
                .map_http()?;
            return Err(e).map_http();
        }
        Ok(config) => config,
    };
    match config.owner_id {
        None => return Err(StatusCode::BAD_REQUEST),
        Some(owner_id) if owner_id != auth.id => return Err(StatusCode::FORBIDDEN),
        _ => {}
    }
    let problem_id = database::problems::create_problem(
        &state.db,
        auth.id,
        config.r#type,
        config.merge_subgroups,
        config.contest_id,
        config.problem_index,
        &config.name,
        config.time_limit_ms,
        config.memory_limit_mb,
        &config.checker_path,
        &config.tests_path,
    )
    .await
    .map_http()?;
    let new_problem_path = PathBuf::from(format!("/problems/{}", problem_id));

    fs::rename(problem_path, new_problem_path)
        .await
        .map_log(TotalVerdict::Bug)
        .map_http()?;

    for (i, subgroup) in config.subgroups.iter().enumerate() {
        database::problems::insert_problem_subgroup(
            &state.db,
            problem_id,
            i,
            &subgroup.r#type,
            &subgroup.tests,
            subgroup.score,
            subgroup.score_per_test,
            &subgroup.depends_on,
        )
        .await
        .map_http()?;
    }

    Ok(Json(problem_id))
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

    log::info!("Verify password");
    let is_valid_password = verify_password(&auth.password_hash, &request.password).map_http()?;

    if !is_valid_password {
        log::error!("Invalid password");
        Err(StatusCode::BAD_REQUEST)
    } else {
        let problem = database::problems::get_problem_by_id(&state.db, problem_id)
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
                .map_log(TotalVerdict::Bug)
                .map_http()?;
            Ok(())
        }
    }
}
