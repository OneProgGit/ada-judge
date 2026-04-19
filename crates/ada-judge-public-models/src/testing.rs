//! Submissions and testing structs

use crate::verdicts::{SubgroupVerdict, TotalVerdict};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Submission's language variants
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::Type))]
#[cfg_attr(
    feature = "db",
    sqlx(type_name = "language", rename_all = "snake_case")
)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    /// clang++ compiler
    Clangpp,
    /// clang compiler
    Clang,
    /// go compiler
    Go,
    /// rustc compiler
    Rust,
    /// Unknown language
    Unknown,
}

/// Returns a file extension for a language
#[must_use]
pub const fn get_language_file_extension(language: &Language) -> &'static str {
    match language {
        Language::Clang => "c",
        Language::Clangpp => "cpp",
        Language::Go => "go",
        Language::Rust => "rs",
        Language::Unknown => "!!",
    }
}

/// Submission request data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubmissonRequest {
    /// Target problem's id
    pub problem_id: i64,
    /// Submission's language
    pub language: Language,
}

/// Total testing result
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct TotalResult {
    /// Total submission's testing verdict
    pub total_verdict: TotalVerdict,
    /// Total submission's score
    pub total_score: i32,
}

/// Subgroup result, including verdict, test of that verdict, score and checker's message
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct SubgroupResult {
    /// Subgroup's verdict
    pub subgroup_verdict: SubgroupVerdict,
    /// Last tested test
    pub test: i32,
    /// Score for the subgroup
    pub score: i32,
}

/// Submission data
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "db", derive(sqlx::FromRow))]
pub struct Submission {
    /// Submission's id
    pub id: i64,
    /// Problem's id
    pub problem_id: i64,
    /// User's id
    pub user_id: i64,
    /// Submission's language
    pub language: Language,
    /// Total submission's testing verdict
    pub total_verdict: TotalVerdict,
    /// Total submission's score
    pub total_score: i32,
    /// Created at timestamp
    pub created_at: DateTime<Utc>,
    /// Subgroup's results
    pub subgroups_results: Vec<SubgroupResult>,
}
