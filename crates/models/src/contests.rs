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

/// Request for creating a contest
#[derive(Clone, Debug, Deserialize)]
pub struct CreateContestRequest {
    /// Contest's name
    pub name: String,
    /// Timestamp of contest beginning
    pub starts_at: DateTime<Utc>,
    /// Timestamp of contest ending
    pub ends_at: DateTime<Utc>,
}
