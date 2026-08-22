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
            Self::C => "c",
            Self::Cpp => "cpp",
            Self::Go => "go",
            Self::Rust => "rs",
            Self::Python => "py",
            Self::FreePascal => "pas",
            Self::Unknown => "!!",
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
    pub verdict: TestingVerdict,
    pub score: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct SubgroupResult {
    pub verdict: Verdict,
    pub test: i32,
    pub score: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct TestResult {
    pub verdict: Verdict,
    pub score: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct Submission {
    pub id: i64,
    pub problem_id: i64,
    pub user_id: i64,
    pub user_login: String,
    pub language: Language,
    pub verdict: TestingVerdict,
    pub score: f64,
    pub subgroups_results: Vec<SubgroupResult>,
    pub tests_results: Vec<TestResult>,
    pub created_at: DateTime<Utc>,
}
