//! Structs for contests

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Leaderboard single row
#[derive(Clone, Debug, Serialize, sqlx::FromRow)]
pub struct LeaderboardRow {
    /// User id
    pub user_id: i64,
    /// Max scores for each problem
    pub scores: Vec<i32>,
    /// Total score
    pub total_score: i64,
}

/// Request for creating/updating a contest
#[derive(Clone, Debug, Deserialize)]
pub struct ContestRequest {
    /// Contest's name
    pub name: String,
    /// Timestamp of contest beginning
    pub starts_at: DateTime<Utc>,
    /// Timestamp of contest ending
    pub ends_at: DateTime<Utc>,
}

/// Contest's config
pub struct ContestConfig {
    /// Contest's owner's user id (optional)
    pub owner_id: Option<i64>,
    /// Contest's name
    pub name: String,
    /// Url to contest's statements
    pub statements_url: String,
    /// Timestamp of contest beginning
    pub starts_at: DateTime<Utc>,
    /// Timestamp of contest ending
    pub ends_at: DateTime<Utc>,
}

/// Contest config visible to all users
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublicContestConfig {
    /// Contest's id
    pub id: i64,
    /// Contest's owner's user id (optional)
    pub owner_id: Option<i64>,
    /// Contest's name
    pub name: String,
    /// Url to contest's statements
    pub statements_url: String,
    /// Timestamp of contest beginning
    pub starts_at: DateTime<Utc>,
    /// Timestamp of contest ending
    pub ends_at: DateTime<Utc>,
}
