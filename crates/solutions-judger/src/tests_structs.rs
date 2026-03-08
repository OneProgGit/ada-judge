use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::enums::AdaJudgeVerdict;

/// Submission data
#[derive(Serialize, Deserialize)]
pub struct Submission {
    pub problem_path: PathBuf,
    pub run_path: PathBuf,
}

/// Result of testing, including result for each subgroup and total score
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TestingResult {
    pub groups_result: Vec<GroupResult>,
    pub total_score: u8,
}

/// Subgroup result, including verdict, test of that verdict, score and checker's message
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GroupResult {
    pub verdict: AdaJudgeVerdict,
    pub test: u8,
    pub score: u8,
    pub checker_msg: String,
}

/// Checker result, including checker's verdict and message
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct CheckerResult {
    pub verdict: AdaJudgeVerdict,
    pub checker_msg: String,
}

/// Useful paths for testing
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct TestsPaths {
    pub output: PathBuf,
    pub error: PathBuf,
    pub solution: PathBuf,
    pub checker: PathBuf,
    pub tests: PathBuf,
}

impl TestsPaths {
    pub fn new(run_path: &Path) -> Self {
        Self {
            output: run_path.join("stdout"),
            error: run_path.join("stderr"),
            solution: run_path.join("run"),
            checker: run_path.join("checker"),
            tests: run_path.join("tests"),
        }
    }
}
