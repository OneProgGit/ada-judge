//! Structs used for testings

use ada_judge_public_models::{
    problems::ProblemConfig,
    testing::{Language, SubgroupResult, Submission, get_language_file_extension},
    verdicts::TotalVerdict,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::Json};
use std::path::{Path, PathBuf};

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
    /// Submission's language
    pub language: Language,
}

/// Submission data for database
#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct DatabaseSubmission {
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
    pub subgroups_results: Json<Vec<SubgroupResult>>,
}

impl From<DatabaseSubmission> for Submission {
    fn from(value: DatabaseSubmission) -> Self {
        Self {
            id: value.id,
            problem_id: value.problem_id,
            user_id: value.user_id,
            language: value.language,
            total_verdict: value.total_verdict,
            total_score: value.total_score,
            created_at: value.created_at,
            subgroups_results: value.subgroups_results.0,
        }
    }
}

/// Useful paths for testing
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TestsPaths {
    /// Path to stdin
    pub input: PathBuf,
    /// Path to stdout
    pub output: PathBuf,
    /// Path to solution binary
    pub solution: PathBuf,
    /// Path to solution source file
    pub solution_source: PathBuf,
    /// Path to checker binary
    pub checker: PathBuf,
    /// Path to the directory, which contains directories with tests inputs and outputs
    pub tests: PathBuf,
    /// Path to the FIFO directory (for interactive problems)
    pub fifo: PathBuf,
}

impl TestsPaths {
    /// Create new test's path based on the run path and config
    #[must_use]
    pub fn new(
        problem_path: &Path,
        run_path: &Path,
        config: &ProblemConfig,
        lang: &Language,
    ) -> Self {
        Self {
            input: run_path.join("stdin"),
            output: run_path.join("stdout"),
            solution: run_path.join("run"),
            solution_source: run_path.join(format!("run.{}", get_language_file_extension(lang))),
            checker: problem_path.join(config.checker_path.clone()),
            tests: problem_path.join(config.tests_path.clone()),
            fifo: run_path.join("fifo"),
        }
    }
}
