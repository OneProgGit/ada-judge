//! Structs for contests

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Leaderboard single row
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct LeaderboardRow {
    /// User id
    pub user_id: i64,
    /// Max scores for each problem
    pub scores: Vec<i32>,
    /// Total score
    pub total_score: i64,
}

/// Request for creating/updating a contest
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContestRequest {
    /// Contest's name
    pub name: String,
    /// Timestamp of contest beginning
    pub starts_at: DateTime<Utc>,
    /// Timestamp of contest ending
    pub ends_at: DateTime<Utc>,
    /// Statements url (ru)
    pub statements_url_ru: String,
    /// Editorial url (ru)
    pub editorial_url_ru: String,
    /// Statements url (en)
    pub statements_url_en: String,
    /// Editorial url (en)
    pub editorial_url_en: String,
    /// Is contest hidden
    pub hidden: bool,
    /// Is upsolving opened
    pub upsolving_opened: bool,
    /// Hide solutions' files
    pub hide_solutions: bool,
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
    /// Statements url (ru)
    pub statements_url_ru: String,
    /// Editorial url (ru)
    pub editorial_url_ru: String,
    /// Statements url (en)
    pub statements_url_en: String,
    /// Editorial url (en)
    pub editorial_url_en: String,
    /// Timestamp of contest beginning
    pub starts_at: DateTime<Utc>,
    /// Timestamp of contest ending
    pub ends_at: DateTime<Utc>,
    /// Is contest hidden
    pub hidden: bool,
    /// Is upsolving opened
    pub upsolving_opened: bool,
    /// Hide solutions' files
    pub hide_solutions: bool,
}
