use crate::{app_state::AppState, middleware::auth::Auth, tools::is_allowed};
use ada_judge_public_models::problems::PublicProblemConfig;
use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use database::problems::get_all_user_problems;
use tools::map::MapHttpExt;

pub async fn get_problems(State(state): State<AppState>) -> Result<Json<Vec<i64>>, StatusCode> {
    Ok(Json(get_all_user_problems(&state.db, -1).await.map_http()?))
}

pub async fn get_my_problems(
    State(state): State<AppState>,
    Auth(auth): Auth,
) -> Result<Json<Vec<i64>>, StatusCode> {
    Ok(Json(
        get_all_user_problems(&state.db, auth.id).await.map_http()?,
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
