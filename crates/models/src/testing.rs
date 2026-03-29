use crate::verdicts::{SubgroupVerdict, TotalVerdict};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use std::path::{Path, PathBuf};

/// Submission language variants
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Language {
    Clang,
    Go,
    Rust,
}

/// Returns a file extension for a language
pub const fn get_lang_str(lang: &Language) -> &'static str {
    match lang {
        Language::Clang => "cpp",
        Language::Go => "go",
        Language::Rust => "rs",
    }
}

/// Submission data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Submission {
    pub problem_id: i64,
    pub lang: Language,
}

/// Submission task data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubmissionTask {
    pub problem_id: i64,
    pub id: i64,
    pub run_dir: PathBuf,
    pub lang: Language,
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
    pub fn new(run_path: &Path, lang: &Language) -> Self {
        Self {
            output: run_path.join("stdout"),
            error: run_path.join("stderr"),
            solution: run_path.join("run"),
            solution_source: run_path.join(format!("run.{}", get_lang_str(lang))),
            checker: run_path.join("checker"),
            tests: run_path.join("tests"),
        }
    }
}
