use std::path::PathBuf;

use crate::{api::ApiError, app_state::AppState, crypt::verify_password, middleware::auth::Auth};
use aj_models::{
    DeletionRequest,
    errors::{AdaJudgeError, Deletion},
    users::{AdminLevel, PrivateUserData, PublicUserData, UsersEvent},
};
use axum::{
    Json,
    extract::{
        Path, State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
};
use futures_util::{SinkExt, StreamExt};
use tokio::{fs, sync::broadcast};
use tools::map::MapHttpExt;

pub async fn my_user_ws(
    ws: WebSocketUpgrade,
    Auth(auth): Auth,
    State(state): State<AppState>,
) -> Result<Response, ApiError> {
    Ok(ws.on_upgrade(move |socket| handle_users_socket(socket, state, Some(auth.id))))
}

pub async fn user_ws(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
) -> Result<Response, ApiError> {
    Ok(ws.on_upgrade(move |socket| handle_users_socket(socket, state, Some(user_id))))
}

pub async fn users_ws(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> Result<Response, ApiError> {
    Ok(ws.on_upgrade(move |socket| handle_users_socket(socket, state, None)))
}

async fn handle_users_socket(socket: WebSocket, state: AppState, user_id: Option<i64>) {
    let (mut ws_tx, mut ws_rx) = socket.split();
    let users_tx = state
        .users_subs
        .get(&user_id)
        .map_or_else(
            || {
                let (tx, _rx) = broadcast::channel(256);
                state.users_subs.insert(user_id, tx.clone());
                tx
            },
            |tx| tx.value().clone(),
        )
        .clone();
    let mut users_rx = users_tx.subscribe();

    let mut send_task = tokio::spawn(async move {
        loop {
            let event = tokio::select! {
                res = users_rx.recv() => match res {
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
    let empty = users_tx.receiver_count() == 0;
    drop(users_tx);
    if empty {
        state.users_subs.remove(&user_id);
    }
}

pub async fn get_public_user_profile(
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
) -> Result<Json<PublicUserData>, ApiError> {
    Ok(Json(
        database::users::get_user_by_id(&state.db, user_id)
            .await
            .map_http()?
            .into(),
    ))
}

pub async fn get_my_user_profile(
    State(state): State<AppState>,
    Auth(auth): Auth,
) -> Result<Json<PrivateUserData>, ApiError> {
    Ok(Json(
        database::users::get_user_by_id(&state.db, auth.id)
            .await
            .map_http()?
            .into(),
    ))
}

pub async fn get_users(
    State(state): State<AppState>,
) -> Result<Json<Vec<PrivateUserData>>, ApiError> {
    Ok(Json(
        database::users::get_users(&state.db).await.map_http()?,
    ))
}

pub async fn get_private_user_profile(
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
) -> Result<Json<PrivateUserData>, ApiError> {
    Ok(Json(
        database::users::get_user_by_id(&state.db, user_id)
            .await
            .map_http()?
            .into(),
    ))
}

pub async fn delete_user_account(
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
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
        let submissions = database::submissions::get_user_submissions(&state.db, user_id)
            .await
            .map_http()?;
        for submission in submissions {
            let submission_id = submission.id;
            fs::remove_dir_all(PathBuf::from(format!("/submissions_envs/{submission_id}")))
                .await
                .map_err(|_| AdaJudgeError::Internal)
                .map_http()?;
        }

        database::users::delete_user(&state.db, user_id)
            .await
            .map_http()?;
        state
            .users_subs
            .get(&None)
            .map(|tx| tx.send(UsersEvent::UserDeleted(user_id)));
        state
            .users_subs
            .get(&Some(user_id))
            .map(|tx| tx.send(UsersEvent::UserDeleted(user_id)));
        Ok(())
    } else {
        Err(AdaJudgeError::Deletion(Deletion::InvalidLoginOrPassword)).map_http()?
    }
}

pub async fn update_user_admin_level(
    State(state): State<AppState>,
    Path(user_id): Path<i64>,
    Json(admin_level): Json<AdminLevel>,
) -> Result<(), ApiError> {
    database::users::change_admin_level(&state.db, user_id, &admin_level)
        .await
        .map_http()?;
    state
        .users_subs
        .get(&None)
        .map(|tx| tx.send(UsersEvent::UserUpdated(user_id)));
    state
        .users_subs
        .get(&Some(user_id))
        .map(|tx| tx.send(UsersEvent::UserUpdated(user_id)));
    Ok(())
}
