//! Contests' config structs

use ada_judge_public_models::contests::PublicContestConfig;
use chrono::{DateTime, Utc};

/// Contest's config for database
#[derive(Clone, Debug, sqlx::FromRow)]
pub struct DatabaseContestConfig {
    /// Contest's id
    pub id: i64,
    /// Contest's owner's user id (optional)
    pub owner_id: Option<i64>,
    /// Contest's name
    pub name: String,
    /// Url to contest's statements
    pub statements_url: String,
    /// Url to contest's editorial
    pub editorial_url: String,
    /// Timestamp of contest beginning
    pub starts_at: DateTime<Utc>,
    /// Timestamp of contest ending
    pub ends_at: DateTime<Utc>,
    /// Timestamp of contest creating
    pub created_at: DateTime<Utc>,
    /// Is contest hidden
    pub hidden: bool,
    /// Is upsolving opened
    pub upsolving_opened: bool,
    /// Hide solutions' files
    pub hide_solutions: bool,
}

impl From<DatabaseContestConfig> for PublicContestConfig {
    fn from(value: DatabaseContestConfig) -> Self {
        Self {
            id: value.id,
            owner_id: value.owner_id,
            name: value.name,
            statements_url: value.statements_url,
            editorial_url: value.editorial_url,
            starts_at: value.starts_at,
            ends_at: value.ends_at,
            hidden: value.hidden,
            upsolving_opened: value.upsolving_opened,
            hide_solutions: value.hide_solutions,
        }
    }
}
