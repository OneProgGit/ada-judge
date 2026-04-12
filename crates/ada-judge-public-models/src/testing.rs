//! Submissions and testing structs

use crate::verdicts::{SubgroupVerdict, TotalVerdict};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Submission's language variants
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Language {
    /// clang++ compiler
    Clang,
    /// go compiler
    Go,
    /// rustc compiler
    Rust,
}

/// Submission request data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubmissonRequest {
    /// Target problem's id
    pub problem_id: i64,
    /// Submission's file language
    pub lang: Language,
}

/// Total testing result
#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct TotalResult {
    /// Total submission's testing verdict
    pub total_verdict: TotalVerdict,
    /// Total submission's score
    pub total_score: i32,
}

/// Subgroup result, including verdict, test of that verdict, score and checker's message
#[derive(Clone, Debug, Default, Serialize, Deserialize, sqlx::FromRow)]
pub struct SubgroupResult {
    /// Subgroup's verdict
    pub subgroup_verdict: SubgroupVerdict,
    /// Last tested test
    pub test: i32,
    /// Score for the subgroup
    pub score: i32,
}

/// Submission data
#[derive(Clone, Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Submission {
    /// Submission's id
    pub id: i64,
    /// Problem's id
    pub problem_id: i64,
    /// User's id
    pub user_id: i64,
    /// Total submission's testing verdict
    pub total_verdict: TotalVerdict,
    /// Total submission's score
    pub total_score: i32,
    /// Created at timestamp
    pub created_at: DateTime<Utc>,
    /// Subgroup's results
    pub subgroups_results: Vec<SubgroupResult>,
}
