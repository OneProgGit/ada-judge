#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![deny(warnings)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

pub mod contests;
pub mod errors;
pub mod problems;
pub mod testing;
pub mod users;
pub mod verdicts;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeletionRequest {
    pub login: String,
    pub password: String,
    pub deletion_confirmation: bool,
}
