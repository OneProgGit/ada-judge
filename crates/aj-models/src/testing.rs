use crate::verdicts::{TestingVerdict, Verdict};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(
    feature = "sqlx",
    sqlx(type_name = "language", rename_all = "snake_case")
)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    C,
    Cpp,
    Go,
    Rust,
    Python,
    FreePascal,
    Unknown,
}

impl Language {
    #[must_use]
    pub const fn file_ext(&self) -> &'static str {
        match &self {
            Language::C => "c",
            Language::Cpp => "cpp",
            Language::Go => "go",
            Language::Rust => "rs",
            Language::Python => "py",
            Language::FreePascal => "pas",
            Language::Unknown => "!!",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubmissonRequest {
    pub language: Language,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct TestingResult {
    pub total_verdict: TestingVerdict,
    pub total_score: i32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct SubgroupResult {
    pub verdict: Verdict,
    pub test: i32,
    pub score: i32,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct TestResult {
    pub verdict: Verdict,
    pub score: Option<i32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct Submission {
    pub id: i64,
    pub problem_id: i64,
    pub user_id: i64,
    pub language: Language,
    pub verdict: TestingVerdict,
    pub score: i32,
    pub subgroups_results: Vec<SubgroupResult>,
    pub tests_results: Vec<TestResult>,
    pub created_at: DateTime<Utc>,
}
