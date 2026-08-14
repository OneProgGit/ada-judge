use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Error {
    #[error("invalid username or password")]
    InvalidUsernameOrPassword,

    #[error("user already exists")]
    AlreadyExists,

    #[error("not found")]
    NotFound,

    #[error("internal error")]
    Internal,
}
