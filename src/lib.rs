//! Judgement system made with Rust.

use std::{
    fs::{self, File, read_to_string},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Duration,
};

use anyhow::{Ok, Result, anyhow};
use fs_extra::dir::CopyOptions;
use wait_timeout::ChildExt;

use crate::{constants::*, problem_config::ProblemConfig, tests_structs::*, verdicts::Verdict};

pub mod constants;
pub mod problem_config;
pub mod tests_structs;
pub mod verdicts;

fn prepare_test_env(
    problem_path: PathBuf,
    config: &ProblemConfig,
    tests_paths: &TestsPaths,
) -> Result<()> {
    fs::copy(
        problem_path.join(config.checker.path.clone()),
        tests_paths.checker.clone(),
    )?;

    let mut opt = CopyOptions::new();
    opt.overwrite = true;
    opt.copy_inside = true;
    opt.content_only = false;

    fs_extra::dir::copy(
        problem_path.join(config.tests.path.clone()),
        tests_paths.tests.clone(),
        &opt,
    )?;

    fs::write(tests_paths.error.clone(), "")?;

    Ok(())
}

fn run_solution(
    config: &ProblemConfig,
    run_path: &Path,
    input_path: &Path,
    tests_paths: &TestsPaths,
) -> Result<Verdict> {
    let stdin_file = File::open(input_path)?;
    let stdout_file = File::create(tests_paths.output.clone())?;
    let stderr_file = File::create(tests_paths.error.clone())?;

    let mut solution_cmd = Command::new("nsjail")
        .args([
            "--mode",
            "o",
            "--rlimit_as",
            &config.limits.memory_limit_mb.to_string(),
            "--disable_proc",
            "--iface_no_lo",
            "--chroot",
            "/",
            "--user",
            "99999",
            "--group",
            "99999",
            "--",
            tests_paths
                .solution
                .to_str()
                .ok_or_else(|| anyhow!("Invalid solution path"))?,
        ])
        .stdin(Stdio::from(stdin_file))
        .stdout(Stdio::from(stdout_file))
        .stderr(Stdio::from(stderr_file))
        .spawn()?;

    let timeout = Duration::from_millis(config.limits.time_limit_ms);
    let solution_status = solution_cmd.wait_timeout(timeout)?;

    _ = solution_cmd.kill();
    match solution_status {
        None => Ok(Verdict::TimeLimitExceeded),
        Some(status) => match status.code() {
            Some(0) => Ok(Verdict::Ok),
            Some(VERDICT_MLE) => Ok(Verdict::MemoryLimitExceeded),
            _ => Ok(Verdict::RuntimeError),
        },
    }
}

fn run_checker(
    config: &ProblemConfig,
    input_path: &Path,
    answer_path: PathBuf,
    tests_paths: &TestsPaths,
) -> Result<CheckerResult> {
    let stderr_file = File::create(tests_paths.error.clone())?;

    let mut checker_cmd = Command::new(tests_paths.checker.clone())
        .args([
            input_path.as_os_str(),
            tests_paths.output.as_os_str(),
            answer_path.as_os_str(),
        ])
        .stderr(Stdio::from(stderr_file))
        .spawn()?;

    let timeout = Duration::from_millis(config.limits.time_limit_ms);
    let checker_status = checker_cmd.wait_timeout(timeout)?;

    _ = checker_cmd.kill();
    match checker_status {
        None => Err(anyhow!("Failed to check solution: checker stucks")),
        Some(status) => {
            let checker_msg = fs::read_to_string(tests_paths.error.clone())?;

            match status.code() {
                Some(CHECKER_OK) => Ok(CheckerResult {
                    verdict: Verdict::Ok,
                    checker_msg,
                }),
                Some(CHECKER_WA) => Ok(CheckerResult {
                    verdict: Verdict::WrongAnswer,
                    checker_msg,
                }),
                Some(CHECKER_PE) => Ok(CheckerResult {
                    verdict: Verdict::PresentationError,
                    checker_msg,
                }),
                _ => Err(anyhow!("Failed to check solution: checker failed")),
            }
        }
    }
}

fn run_single_test(
    config: &ProblemConfig,
    run_path: &Path,
    tests_paths: &TestsPaths,
    test_id: u8,
) -> Result<CheckerResult> {
    let test_path = tests_paths.tests.join(test_id.to_string());

    let input_path = test_path.join("in");
    let answer_path = test_path.join("out");

    let solution_verdict = run_solution(config, run_path, &input_path, tests_paths)?;

    if solution_verdict != Verdict::Ok {
        return Ok(CheckerResult {
            verdict: solution_verdict,
            checker_msg: String::new(),
        });
    }

    Ok(run_checker(config, &input_path, answer_path, tests_paths)?)
}

/// Test solution and return a verdict for each subgroup.
pub fn test(problem_path: PathBuf, run_path: PathBuf) -> Result<Vec<TestResult>> {
    let config: ProblemConfig = toml::from_str(&read_to_string(problem_path.join("config.toml"))?)?;
    let tests_paths = TestsPaths::new(&run_path);

    prepare_test_env(problem_path, &config, &tests_paths)?;

    let mut result: Vec<TestResult> = Vec::with_capacity(config.test_groups.len());

    for test_group in config.test_groups.clone() {
        let mut test_result = TestResult {
            verdict: Verdict::Ok,
            test: 0,
            checker_msg: String::new(),
        };

        for test_id in test_group.tests {
            let run_result = run_single_test(&config, &run_path, &tests_paths, test_id)?;

            test_result.verdict = run_result.verdict.clone();
            test_result.test = test_id;
            test_result.checker_msg = run_result.checker_msg;

            if run_result.verdict != Verdict::Ok {
                break;
            }
        }

        result.push(test_result);
    }

    Ok(result)
}
