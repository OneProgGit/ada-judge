//! Structs for contests

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

/// Request for getting contest leaderborad
#[derive(Clone, Debug, Deserialize)]
pub struct GetContestLeaderboardRequest {
    /// Contest id
    pub contest_id: i64,
}
