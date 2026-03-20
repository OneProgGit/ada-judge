use crate::verdicts::{SubgroupVerdict, TotalVerdict};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use std::path::{Path, PathBuf};

/// Submission data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Submission {
    pub problem_path: PathBuf,
}

/// Submission task data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubmissionTask {
    pub problem_path: PathBuf,
    pub run_dir: PathBuf,
    pub id: i64,
}

/// Total testing result
#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct TotalResult {
    pub total_verdict: TotalVerdict,
    pub total_score: i32,
}

/// Subgroup result, including verdict, test of that verdict, score and checker's message
#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct GroupResult {
    pub verdict: SubgroupVerdict,
    pub test: i32,
    pub score: i32,
    pub checker_msg: String,
}

/// Checker result, including checker's verdict and message
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CheckerResult {
    pub verdict: SubgroupVerdict,
    pub checker_msg: String,
}

/// Useful paths for testing
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TestsPaths {
    pub output: PathBuf,
    pub error: PathBuf,
    pub solution: PathBuf,
    pub solution_source: PathBuf,
    pub checker: PathBuf,
    pub tests: PathBuf,
}

impl TestsPaths {
    pub fn new(run_path: &Path) -> Self {
        Self {
            output: run_path.join("stdout"),
            error: run_path.join("stderr"),
            solution: run_path.join("run"),
            solution_source: run_path.join("run.rs"),
            checker: run_path.join("checker"),
            tests: run_path.join("tests"),
        }
    }
}
