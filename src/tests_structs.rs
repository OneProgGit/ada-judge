use std::path::{Path, PathBuf};

use crate::verdicts::Verdict;

#[derive(Clone, Debug)]
/// Result of testing, including result for each subgroup and total score
pub struct TestingResult {
    pub groups_result: Vec<GroupResult>,
    pub total_score: u8,
}

/// Subgroup result, including verdict, test of that verdict, score and checker's message
#[derive(Clone, Debug)]
pub struct GroupResult {
    pub verdict: Verdict,
    pub test: u8,
    pub score: u8,
    pub checker_msg: String,
}

/// Checker result, including checker's verdict and message
#[derive(Clone, Debug)]
pub(crate) struct CheckerResult {
    pub verdict: Verdict,
    pub checker_msg: String,
}

/// Useful paths for testing
#[derive(Clone, Debug)]
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
