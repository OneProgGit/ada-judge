//! Structs used for testings

use crate::{
    problem_config::ProblemConfig,
    verdicts::{SubgroupVerdict, TotalVerdict},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use std::path::{Path, PathBuf};

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

/// Returns a file extension for a language
#[must_use]
pub const fn get_lang_str(lang: &Language) -> &'static str {
    match lang {
        Language::Clang => "cpp",
        Language::Go => "go",
        Language::Rust => "rs",
    }
}

/// Submission request data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubmissonRequest {
    /// Target problem's id
    pub problem_id: i64,
    /// Submission's file language
    pub lang: Language,
}

/// Submission task data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubmissionTask {
    /// Target problem's id
    pub problem_id: i64,
    /// Target problem's path
    pub problem_path: PathBuf,
    /// Task's id
    pub id: i64,
    /// Test environment's paths
    pub run_dir: PathBuf,
    /// Submission's file language
    pub lang: Language,
}

/// Total testing result
#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct TotalResult {
    /// Total submission's testing verdict
    pub total_verdict: TotalVerdict,
    /// Total submission's score
    pub total_score: i32,
}

/// Submission data
#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
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
}

/// Subgroup result, including verdict, test of that verdict, score and checker's message
#[derive(Clone, Debug, Default, Serialize, Deserialize, FromRow)]
pub struct SubgroupResult {
    /// Subgroup's verdict
    pub subgroup_verdict: SubgroupVerdict,
    /// Last tested test
    pub test: i32,
    /// Score for the subgroup
    pub score: i32,
    /// Checker's stderr message
    pub checker_msg: String,
}

/// Checker result, including checker's verdict and message
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CheckerResult {
    /// Checker's verdict
    pub verdict: SubgroupVerdict,
    /// Checker's stderr message
    pub checker_msg: String,
}

/// Useful paths for testing
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TestsPaths {
    /// Path to stdout
    pub output: PathBuf,
    /// Path to stderr
    pub error: PathBuf,
    /// Path to solution binary
    pub solution: PathBuf,
    /// Path to solution source file
    pub solution_source: PathBuf,
    /// Path to checker binary
    pub checker: PathBuf,
    /// Path to the directory, which contains directories with tests inputs and outputs
    pub tests: PathBuf,
}

impl TestsPaths {
    /// Create new test's path based on the run path and config
    #[must_use]
    pub fn new(run_path: &Path, config: &ProblemConfig, lang: &Language) -> Self {
        Self {
            output: run_path.join("stdout"),
            error: run_path.join("stderr"),
            solution: run_path.join("run"),
            solution_source: run_path.join(format!("run.{}", get_lang_str(lang))),
            checker: run_path.join(config.checker_path.clone()),
            tests: run_path.join(config.tests_path.clone()),
        }
    }
}
