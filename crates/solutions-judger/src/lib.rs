use crate::{constants::*, problem_config::ProblemConfig};
use apalis::prelude::TaskSink;
use axum::{Json, extract::State};
use fs_extra::dir::CopyOptions;
use models::AppState;
use models::{
    enums::{AdaJudgeError, AdaJudgeVerdict},
    testing::*,
};
use std::{
    fs::{self, File, read_to_string},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::Arc,
    time::Duration,
};
use wait_timeout::ChildExt;

pub mod constants;
pub mod problem_config;

fn prepare_test_env(
    problem_path: PathBuf,
    config: &ProblemConfig,
    tests_paths: &TestsPaths,
) -> Result<(), AdaJudgeError> {
    fs::copy(
        problem_path.join(config.checker.path.clone()),
        tests_paths.checker.clone(),
    )
    .map_err(|_| AdaJudgeError::InvalidProblem)?;

    let mut opt = CopyOptions::new();
    opt.overwrite = true;
    opt.copy_inside = true;
    opt.content_only = false;

    fs_extra::dir::copy(
        problem_path.join(config.tests.path.clone()),
        tests_paths.tests.clone(),
        &opt,
    )
    .map_err(|_| AdaJudgeError::InvalidProblem)?;

    fs::write(tests_paths.error.clone(), "").map_err(|_| AdaJudgeError::InvalidProblem)?;

    Ok(())
}

fn run_solution(
    config: &ProblemConfig,
    input_path: &Path,
    tests_paths: &TestsPaths,
) -> Result<AdaJudgeVerdict, AdaJudgeError> {
    let stdin_file = File::open(input_path).map_err(|_| AdaJudgeError::InvalidProblem)?;
    let stdout_file =
        File::create(tests_paths.output.clone()).map_err(|_| AdaJudgeError::InvalidProblem)?;
    let stderr_file =
        File::create(tests_paths.error.clone()).map_err(|_| AdaJudgeError::InvalidProblem)?;

    let mut solution_cmd = Command::new("docker")
        .args([
            "run",
            "--rm",
            "--network",
            "none",
            "--memory",
            &format!("{}m", config.limits.memory_limit_mb),
            "--cpus",
            "0.3",
            "--pids-limit",
            "32",
            "--read-only",
            "--cap-drop",
            "ALL",
            "-i",
            "--security-opt",
            "no-new-privileges",
            "-v",
            &format!("{}:/sandbox/bin:ro", tests_paths.solution.display()),
            "-w",
            "/sandbox",
            "sandbox-runner",
            "/sandbox/bin",
        ])
        .stdin(Stdio::from(stdin_file))
        .stdout(Stdio::from(stdout_file))
        .stderr(Stdio::from(stderr_file))
        .spawn()
        .map_err(|_| AdaJudgeError::Bug)?;

    let timeout = Duration::from_millis(config.limits.time_limit_ms);
    let solution_status = solution_cmd
        .wait_timeout(timeout)
        .map_err(|_| AdaJudgeError::Bug)?;

    _ = solution_cmd.kill();
    match solution_status {
        None => Ok(AdaJudgeVerdict::TimeLimitExceeded),
        Some(status) => match status.code() {
            Some(VERDICT_OK) => Ok(AdaJudgeVerdict::Ok),
            Some(VERDICT_MLE) => Ok(AdaJudgeVerdict::MemoryLimitExceeded),
            _ => Ok(AdaJudgeVerdict::RuntimeError),
        },
    }
}

fn run_checker(
    config: &ProblemConfig,
    input_path: &Path,
    answer_path: PathBuf,
    tests_paths: &TestsPaths,
) -> Result<CheckerResult, AdaJudgeError> {
    let stderr_file =
        File::create(tests_paths.error.clone()).map_err(|_| AdaJudgeError::InvalidProblem)?;

    let mut checker_cmd = Command::new("docker")
        .args([
            "run",
            "--rm",
            "--network",
            "none",
            "--memory",
            &format!("{}m", config.limits.memory_limit_mb),
            "--cpus",
            "0.3",
            "--pids-limit",
            "32",
            "--read-only",
            "--cap-drop",
            "ALL",
            "-i",
            "--security-opt",
            "no-new-privileges",
            "-v",
            &format!("{}:/sandbox/bin:ro", tests_paths.checker.display()),
            "-v",
            &format!("{}:/sandbox/input:ro", input_path.display()),
            "-v",
            &format!("{}:/sandbox/output:ro", tests_paths.output.display()),
            "-v",
            &format!("{}:/sandbox/answer:ro", answer_path.display()),
            "-w",
            "/sandbox",
            "sandbox-runner",
            "/sandbox/bin",
            "/sandbox/input",
            "/sandbox/output",
            "/sandbox/answer",
        ])
        .stderr(Stdio::from(stderr_file))
        .spawn()
        .map_err(|_| AdaJudgeError::Bug)?;

    let timeout = Duration::from_millis(config.limits.time_limit_ms);
    let checker_status = checker_cmd
        .wait_timeout(timeout)
        .map_err(|_| AdaJudgeError::Bug)?;

    _ = checker_cmd.kill();
    match checker_status {
        None => Err(AdaJudgeError::CheckerFailed),
        Some(status) => {
            let checker_msg = fs::read_to_string(tests_paths.error.clone())
                .map_err(|_| AdaJudgeError::InvalidProblem)?;

            match status.code() {
                Some(CHECKER_OK) => Ok(CheckerResult {
                    verdict: AdaJudgeVerdict::Ok,
                    checker_msg,
                }),
                Some(CHECKER_WA) => Ok(CheckerResult {
                    verdict: AdaJudgeVerdict::WrongAnswer,
                    checker_msg,
                }),
                Some(CHECKER_PE) => Ok(CheckerResult {
                    verdict: AdaJudgeVerdict::PresentationError,
                    checker_msg,
                }),
                _ => Err(AdaJudgeError::CheckerFailed),
            }
        }
    }
}

fn run_single_test(
    config: &ProblemConfig,
    tests_paths: &TestsPaths,
    test_id: u8,
) -> Result<CheckerResult, AdaJudgeError> {
    let test_path = tests_paths.tests.join(test_id.to_string());

    let input_path = test_path.join("in");
    let answer_path = test_path.join("out");

    let solution_verdict = run_solution(config, &input_path, tests_paths)?;

    if solution_verdict != AdaJudgeVerdict::Ok {
        return Ok(CheckerResult {
            verdict: solution_verdict,
            checker_msg: String::default(),
        });
    }

    run_checker(config, &input_path, answer_path, tests_paths)
}

pub fn test(
    Json(submission): Json<Submission>,
) -> Result<Json<TestingResult>, Json<AdaJudgeError>> {
    let problem_path = submission
        .problem_path
        .canonicalize()
        .map_err(|_| AdaJudgeError::InvalidProblem)?;
    let run_path = submission
        .run_path
        .canonicalize()
        .map_err(|_| AdaJudgeError::InvalidProblem)?;

    let config: ProblemConfig = toml::from_str(
        &read_to_string(problem_path.join("config.toml"))
            .map_err(|_| AdaJudgeError::InvalidProblem)?,
    )
    .map_err(|_| AdaJudgeError::InvalidProblem)?;

    for (i, group) in config.test_groups.iter().enumerate() {
        if let Some(depends_on) = group.depends_on.clone() {
            for x in depends_on {
                if x >= i {
                    return Err(AdaJudgeError::InvalidProblem.into());
                }
            }
        }
    }

    let tests_paths = TestsPaths::new(&run_path);

    prepare_test_env(problem_path, &config, &tests_paths)?;

    let mut groups_result: Vec<GroupResult> = Vec::with_capacity(config.test_groups.len());
    let mut total_score = 0;

    for test_group in config.test_groups.clone() {
        let mut test_result = GroupResult {
            verdict: AdaJudgeVerdict::Ok,
            test: 0,
            score: test_group.score,
            checker_msg: String::new(),
        };

        if let Some(depends_on) = test_group.depends_on {
            for i in depends_on {
                if groups_result[i].verdict != AdaJudgeVerdict::Ok {
                    test_result.verdict = AdaJudgeVerdict::Skipped;
                    test_result.score = 0;
                    break;
                }
            }
        }

        if test_result.verdict != AdaJudgeVerdict::Skipped {
            for test_id in test_group.tests {
                let run_result = run_single_test(&config, &tests_paths, test_id)?;

                test_result.verdict = run_result.verdict.clone();
                test_result.test = test_id;
                test_result.checker_msg = run_result.checker_msg;

                if run_result.verdict != AdaJudgeVerdict::Ok {
                    test_result.score = 0;
                    break;
                }
            }
        }

        total_score += test_result.score;
        groups_result.push(test_result);
    }

    Ok(TestingResult {
        groups_result,
        total_score,
    }
    .into())
}

pub async fn push_submission_to_queue(
    State(state): State<Arc<AppState>>,
    Json(submission): Json<Submission>,
) -> Result<(), Json<AdaJudgeError>> {
    state
        .apalis_backend
        .lock()
        .await
        .push(submission)
        .await
        .map_err(|_| AdaJudgeError::Bug)?;
    Ok(())
}
