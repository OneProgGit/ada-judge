//! Contests' config structs

use chrono::{DateTime, Utc};

/// Contest's config
pub struct ContestConfig {
    /// Contest's owner's user id (optional)
    pub owner_id: Option<i64>,
    /// Contest's name
    pub name: String,
    /// Timestamp of contest beginning
    pub starts_at: DateTime<Utc>,
    /// Timestamp of contest ending
    pub ends_at: DateTime<Utc>,
}

/// Contest's config for database
#[derive(Clone, Debug, sqlx::FromRow)]
pub struct DatabaseContestConfig {
    /// Contest's id
    pub id: i64,
    /// Contest's owner's user id (optional)
    pub owner_id: Option<i64>,
    /// Contest's name
    pub name: String,
    /// Timestamp of contest beginning
    pub starts_at: DateTime<Utc>,
    /// Timestamp of contest ending
    pub ends_at: DateTime<Utc>,
    /// Timestamp of contest creating
    pub created_at: DateTime<Utc>,
}
