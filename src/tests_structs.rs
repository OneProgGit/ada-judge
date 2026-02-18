use std::path::{Path, PathBuf};

use crate::verdicts::Verdict;

pub struct TestingResult {
    pub groups_result: Vec<GroupResult>,
    pub total_score: u8,
}

#[derive(Clone)]
pub struct GroupResult {
    pub verdict: Verdict,
    pub test: u8,
    pub score: u8,
    pub checker_msg: String,
}

#[derive(Clone)]
pub(crate) struct CheckerResult {
    pub verdict: Verdict,
    pub checker_msg: String,
}

#[derive(Clone)]
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
            output: run_path.join("stdout.txt"),
            error: run_path.join("stderr.txt"),
            solution: run_path.join("run"),
            checker: run_path.join("checker"),
            tests: run_path.join("tests"),
        }
    }
}
