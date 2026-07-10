//! Public shared models for `ada-judge`

#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![deny(warnings)]
#![deny(missing_docs)]
#![deny(rustdoc::all)]
#![deny(rustdoc::broken_intra_doc_links)]
#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

pub mod contests;
pub mod problems;
pub mod testing;
pub mod users;
pub mod verdicts;

/// Deletion request called from frontend
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeletionRequest {
    /// Login
    pub login: String,
    /// Password
    pub password: String,
    /// Password confirmation
    pub password_confirmation: String,
    /// Deletion confirmation
    pub deletion_confirmation: bool,
}
