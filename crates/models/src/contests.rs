//! Structs for contests

use serde::Serialize;

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
