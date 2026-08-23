use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdaJudgeError {
    #[error("not found")]
    NotFound,

    #[error("internal error")]
    Internal,

    #[error("invalid problem config")]
    InvalidProblem(#[from] InvalidProblem),

    #[error("invalid JWT")]
    InvalidJwt,

    #[error("auth error")]
    Auth(#[from] AuthError),

    #[error("deletion error")]
    Deletion(#[from] Deletion),

    #[error("forbidden")]
    Forbidden,

    #[error("contest error")]
    Contest(#[from] Contest),

    #[error("bad request")]
    BadRequest,
}

#[derive(Error, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AuthError {
    #[error("invalid login or password")]
    InvalidLoginOrPassword,

    #[error("user already exists")]
    AlreadyExists,

    #[error("passwords don't match")]
    PasswordsDontMatch,
}

#[derive(Error, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum InvalidProblem {
    #[error("subgroup {subgroup} depends on subgroup {depends_on}")]
    SubgroupConflict { subgroup: usize, depends_on: usize },

    #[error("subgroup {subgroup} has both score and score_per_test fields (or no of them)")]
    InvalidSubgroupScoring { subgroup: usize },

    #[error("no problem config found")]
    MissingConfig,

    #[error("toml error")]
    TomlError { message: String },

    #[error("owner id doesn't match or missing")]
    OwnerId,
}

#[derive(Error, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Deletion {
    #[error("invalid login or password")]
    InvalidLoginOrPassword,

    #[error("missing deletion confirmation")]
    MissingDeletionConfirmation,
}

#[derive(Error, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Contest {
    #[error("start time is >= finish time")]
    Time,
}
