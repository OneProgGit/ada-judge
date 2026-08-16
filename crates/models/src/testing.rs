use aj_models::{
    problems::ProblemConfig,
    testing::{Language, SubgroupResult, Submission, TestResult},
    verdicts::TestingVerdict,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, types::Json};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SubmissionTask {
    pub problem_id: i64,
    pub problem_path: PathBuf,
    pub id: i64,
    pub run_dir: PathBuf,
    pub language: Language,
}

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct DatabaseSubmission {
    pub id: i64,
    pub problem_id: i64,
    pub user_id: i64,
    pub language: Language,
    pub verdict: TestingVerdict,
    pub score: f64,
    pub subgroups_results: Json<Vec<SubgroupResult>>,
    pub tests_results: Json<Vec<TestResult>>,
    pub created_at: DateTime<Utc>,
}

impl From<DatabaseSubmission> for Submission {
    fn from(value: DatabaseSubmission) -> Self {
        Self {
            id: value.id,
            problem_id: value.problem_id,
            user_id: value.user_id,
            language: value.language,
            verdict: value.verdict,
            score: value.score,
            created_at: value.created_at,
            subgroups_results: value.subgroups_results.0,
            tests_results: value.tests_results.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TestsPaths {
    pub input: PathBuf,
    pub output: PathBuf,
    pub solution: PathBuf,
    pub solution_source: PathBuf,
    pub checker: PathBuf,
    pub tests: PathBuf,
    pub fifo: PathBuf,
}

impl TestsPaths {
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
            solution_source: run_path.join(format!("run.{}", lang.file_ext())),
            checker: problem_path.join(PathBuf::from("checker")),
            tests: problem_path.join(config.tests_path.clone()),
            fifo: run_path.join("fifo"),
        }
    }
}
