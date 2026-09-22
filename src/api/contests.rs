#![allow(clippy::result_large_err)]
#![allow(clippy::option_map_unit_fn)]

use std::path::PathBuf;

use crate::{
    api::ApiError,
    app_state::AppState,
    crypt::verify_password,
    middleware::{auth::Auth, contests::ensure_contest_started_common},
    tools::is_allowed,
};
use aj_models::{
    DeletionRequest,
    contests::{
        ContestEvent, ContestPost, ContestPostRequest, ContestRequest, LeaderboardRow,
        PublicContestConfig,
    },
    errors::{AdaJudgeError, Contest, Deletion},
    problems::{ProblemQuestion, PublicProblemConfig},
    users::AdminLevel,
};
use axum::{
    Json,
    extract::{
        Path, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
};
use chrono::Utc;
use database::contests::GetContestsScope;
use futures_util::{SinkExt, StreamExt};
use tokio::{fs, sync::broadcast};
use tokio_util::sync::CancellationToken;
use tools::map::MapHttpExt;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum ContestsSubScope {
    User,
    NotHidden,
    All,
    Contest(i64),
}

pub async fn contest_ws(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path(contest_id): Path<i64>,
    Auth(auth): Auth,
) -> Result<Response, ApiError> {
    Ok(ws.on_upgrade(move |socket| handle_contests_socket(socket, state, Some(contest_id))))
}

pub async fn my_contests_ws(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Auth(auth): Auth,
) -> Result<Response, ApiError> {
    Ok(ws.on_upgrade(move |socket| handle_contests_socket(socket, state, None)))
}

pub async fn contests_ws(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Auth(auth): Auth,
) -> Result<Response, ApiError> {
    Ok(ws.on_upgrade(move |socket| handle_contests_socket(socket, state, None)))
}

async fn handle_contests_socket(socket: WebSocket, state: AppState, contest_id: Option<i64>) {
    let (mut ws_tx, mut ws_rx) = socket.split();
    let cancel = CancellationToken::new();
    let entry = state
        .contests_subs
        .entry(contest_id)
        .or_insert_with(|| (broadcast::channel(256).0, cancel.clone()));
    let (contests_tx, cancel) = (entry.value().0.clone(), entry.value().1.clone());
    drop(entry);
    let mut contests_rx = contests_tx.subscribe();

    let mut send_task = tokio::spawn(async move {
        loop {
            let event = tokio::select! {
                () = cancel.cancelled() => break,
                res = contests_rx.recv() => match res {
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

    if contests_tx.receiver_count() == 0 {
        state.contests_subs.remove(&contest_id);
    }
}

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
    let scope = if auth.admin_level == AdminLevel::Owner {
        GetContestsScope::All
    } else {
        GetContestsScope::NotHidden(auth.id)
    };
    let now = Utc::now();
    Ok(Json(
        database::contests::get_contests(&state.db, scope)
            .await
            .map_http()?
            .into_iter()
            .map(|mut contest| {
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
                contest
            })
            .collect(),
    ))
}

pub async fn get_my_contests(
    State(state): State<AppState>,
    Auth(auth): Auth,
) -> Result<Json<Vec<PublicContestConfig>>, ApiError> {
    Ok(Json(
        database::contests::get_contests(&state.db, GetContestsScope::User(auth.id))
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
        let id = database::contests::create_contest(&state.db, auth.id, &request)
            .await
            .map_http()?;
        state
            .contests_subs
            .get(&None)
            .map(|tx| tx.0.send(ContestEvent::NewContest(id)));
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
        state
            .contests_subs
            .get(&None)
            .map(|tx| tx.0.send(ContestEvent::ContestUpdated(contest_id)));
        state
            .contests_subs
            .get(&Some(contest_id))
            .map(|tx| tx.0.send(ContestEvent::ContestUpdated(contest_id)));
        Ok(())
    }
}

#[allow(clippy::too_many_lines)]
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
            state
                .contests_subs
                .get(&Some(contest_id))
                .map(|tx| tx.0.send(ContestEvent::ContestDeleted(contest_id)));
            state.contests_subs.remove(&Some(contest_id));
            state.questions_subs.remove(&contest_id);
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
    if !is_allowed(auth.id, contest.owner_id, &auth.admin_level)
        && contest.co_authors.binary_search(&auth.id).is_err()
    {
        return Err(AdaJudgeError::Forbidden).map_http()?;
    }
    let id = database::contests::create_contest_post(&state.db, auth.id, contest_id, &request)
        .await
        .map_http()?;
    state
        .contests_subs
        .get(&Some(contest_id))
        .map(|tx| tx.0.send(ContestEvent::NewPost(id)));
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
    let contest = database::contests::get_contest(&state.db, post.contest_id)
        .await
        .map_http()?;
    if !is_allowed(auth.id, Some(post.owner_id), &auth.admin_level)
        && !is_allowed(auth.id, contest.owner_id, &auth.admin_level)
    {
        return Err(AdaJudgeError::Forbidden).map_http()?;
    }
    database::contests::update_contest_post(&state.db, post_id, &request)
        .await
        .map_http()?;
    state
        .contests_subs
        .get(&Some(post_id))
        .map(|tx| tx.0.send(ContestEvent::PostUpdated(post_id)));

    Ok(())
}

#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
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
        let contest = database::contests::get_contest(&state.db, post.contest_id)
            .await
            .map_http()?;
        if is_allowed(auth.id, Some(post.owner_id), &auth.admin_level)
            || is_allowed(auth.id, contest.owner_id, &auth.admin_level)
        {
            database::contests::delete_contest_post(&state.db, post_id)
                .await
                .map_http()?;
            state
                .contests_subs
                .get(&Some(contest.id))
                .map(|tx| tx.0.send(ContestEvent::PostDeleted(post.id)));
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
    Auth(auth): Auth,
    Path(post_id): Path<i64>,
) -> Result<Json<ContestPost>, ApiError> {
    let post = database::contests::get_contest_post(&state.db, post_id)
        .await
        .map_http()?;

    if ensure_contest_started_common(&state.db, auth.id, post.contest_id, auth.admin_level)
        .await
        .is_err()
    {
        return Err(AdaJudgeError::Forbidden).map_http()?;
    }

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
    if ensure_contest_started_common(&state.db, auth.id, contest_id, auth.admin_level)
        .await
        .is_err()
    {
        return Err(AdaJudgeError::Forbidden).map_http()?;
    }

    Ok(Json(
        database::contests::get_contest_posts(&state.db, contest_id)
            .await
            .map_http()?,
    ))
}

pub async fn get_all_contest_problems_questions(
    State(state): State<AppState>,
    Auth(auth): Auth,
    Path(contest_id): Path<i64>,
) -> Result<Json<Vec<ProblemQuestion>>, ApiError> {
    let contest = database::contests::get_contest(&state.db, contest_id)
        .await
        .map_http()?;
    if !is_allowed(auth.id, contest.owner_id, &auth.admin_level)
        && contest.co_authors.binary_search(&auth.id).is_err()
    {
        return Err(AdaJudgeError::Forbidden).map_http()?;
    }

    Ok(Json(
        database::contests::get_problems_questions(&state.db, None, contest_id)
            .await
            .map_http()?,
    ))
}

pub async fn get_my_contest_problems_questions(
    State(state): State<AppState>,
    Auth(auth): Auth,
    Path(contest_id): Path<i64>,
) -> Result<Json<Vec<ProblemQuestion>>, ApiError> {
    if ensure_contest_started_common(&state.db, auth.id, contest_id, auth.admin_level)
        .await
        .is_err()
    {
        return Err(AdaJudgeError::Forbidden).map_http()?;
    }

    Ok(Json(
        database::contests::get_problems_questions(&state.db, Some(auth.id), contest_id)
            .await
            .map_http()?,
    ))
}
