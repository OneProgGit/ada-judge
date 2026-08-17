use aj_models::errors::AdaJudgeError;
use axum::{Json, http::StatusCode};

pub mod auth;
pub mod contests;
pub mod problems;
pub mod submissions;
pub mod users;

pub type ApiError = (StatusCode, Json<AdaJudgeError>);
