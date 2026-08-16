use crate::{
    constants::{CHECKER_OK, CHECKER_PE, CHECKER_WA},
    tools::convert_path_in_container_to_path_in_host,
};
use aj_models::{
    problems::ProblemConfig,
    verdicts::{TestingVerdict, Verdict},
};
use models::testing::TestsPaths;
use std::{env, path::Path};
use tokio::process::Command;

#[allow(clippy::cast_sign_loss)]
pub async fn get_checker_verdict(
    config: &ProblemConfig,
    input_path: &Path,
    answer_path: &Path,
) -> Result<Verdict, TestingVerdict> {
    let sandbox_image = env::var("SANDBOX_IMAGE").map_err(|_| TestingVerdict::Fail)?;

    let checker_cmd = Command::new("docker")
        .args([
            "run",
            "--rm",
            "--init",
            "--network",
            "none",
            "--memory",
            &format!("{}m", config.memory_limit_mb),
            "--ulimit",
            "stack=67108864:-1",
            "--cpus",
            "0.5",
            "--pids-limit",
            "32",
            "--read-only",
            "--cap-drop",
            "ALL",
            "-i",
            "--security-opt",
            "no-new-privileges",
            "-v",
            &format!(
                "{}:/sandbox/bin:ro",
                convert_path_in_container_to_path_in_host(&tests_paths.checker)?.display()
            ),
            "-v",
            &format!(
                "{}:/sandbox/input:ro",
                convert_path_in_container_to_path_in_host(input_path)?.display()
            ),
            "-v",
            &format!(
                "{}:/sandbox/output:ro",
                convert_path_in_container_to_path_in_host(&tests_paths.output)?.display()
            ),
            "-v",
            &format!(
                "{}:/sandbox/answer:ro",
                convert_path_in_container_to_path_in_host(&answer_path)?.display()
            ),
            "-w",
            "/sandbox",
            &sandbox_image,
            "timeout",
            "-s",
            "KILL",
            &format!("{}s", f64::from(config.time_limit_ms) / 1000.),
            "/sandbox/bin",
            "/sandbox/input",
            "/sandbox/output",
            "/sandbox/answer",
        ])
        .status();

    let checker_status = checker_cmd.await;
    checker_status.map_or(Err(TestingVerdict::Fail), |status| match status.code() {
        Some(CHECKER_OK) => Ok(Verdict::Ok),
        Some(CHECKER_WA) => Ok(Verdict::WrongAnswer),
        Some(CHECKER_PE) => Ok(Verdict::PresentationError),
        _ => Err(TestingVerdict::Fail),
    })
}

#[allow(clippy::cast_sign_loss)]
pub async fn get_checker_run_twice_verdict(
    config: &ProblemConfig,
    answer_path: &Path,
    tests_paths: &TestsPaths,
    stage: i32,
) -> Result<Verdict, TestingVerdict> {
    let sandbox_image = env::var("SANDBOX_IMAGE").map_err(|_| TestingVerdict::Bug)?;

    let checker_cmd = Command::new("docker")
        .args([
            "run",
            "--rm",
            "--init",
            "--network",
            "none",
            "--memory",
            &format!("{}m", config.memory_limit_mb),
            "--ulimit",
            "stack=67108864:-1",
            "--cpus",
            "0.5",
            "--pids-limit",
            "32",
            "--read-only",
            "--cap-drop",
            "ALL",
            "-i",
            "--security-opt",
            "no-new-privileges",
            "-v",
            &format!(
                "{}:/sandbox/bin:ro",
                convert_path_in_container_to_path_in_host(&tests_paths.checker)?.display()
            ),
            "-v",
            &format!(
                "{}:/sandbox/input:ro",
                convert_path_in_container_to_path_in_host(&tests_paths.output)?.display()
            ),
            "-v",
            &format!(
                "{}:/sandbox/output",
                convert_path_in_container_to_path_in_host(&tests_paths.input)?.display()
            ),
            "-v",
            &format!(
                "{}:/sandbox/answer:ro",
                convert_path_in_container_to_path_in_host(answer_path)?.display()
            ),
            "-w",
            "/sandbox",
            &sandbox_image,
            "timeout",
            "-s",
            "KILL",
            &format!("{}s", f64::from(config.time_limit_ms) / 1000.),
            "/sandbox/bin",
            "/sandbox/input",
            "/sandbox/output",
            "/sandbox/answer",
            &stage.to_string(),
        ])
        .status();

    let checker_status = checker_cmd.await;
    checker_status.map_or(Err(TestingVerdict::Fail), |status| match status.code() {
        Some(CHECKER_OK) => Ok(Verdict::Ok),
        Some(CHECKER_WA) => Ok(Verdict::WrongAnswer),
        Some(CHECKER_PE) => Ok(Verdict::PresentationError),
        _ => Err(TestingVerdict::Fail),
    })
}

#[allow(clippy::cast_sign_loss)]
pub async fn get_checker_run_twice_interactive_verdict(
    config: &ProblemConfig,
    answer_path: &Path,
    final_output: &Path,
    tests_paths: &TestsPaths,
    stage: i32,
) -> Result<Verdict, TestingVerdict> {
    let sandbox_image = env::var("SANDBOX_IMAGE").map_err(|_| TestingVerdict::Bug)?;

    let checker_cmd = Command::new("docker")
        .args([
            "run",
            "--rm",
            "--init",
            "--network",
            "none",
            "--memory",
            &format!("{}m", config.memory_limit_mb),
            "--ulimit",
            "stack=67108864:-1",
            "--cpus",
            "0.5",
            "--pids-limit",
            "32",
            "--read-only",
            "--cap-drop",
            "ALL",
            "-i",
            "--security-opt",
            "no-new-privileges",
            "-v",
            &format!(
                "{}:/sandbox/bin:ro",
                convert_path_in_container_to_path_in_host(&tests_paths.checker)?.display()
            ),
            "-v",
            &format!(
                "{}:/sandbox/input:ro",
                convert_path_in_container_to_path_in_host(&tests_paths.output)?.display()
            ),
            "-v",
            &format!(
                "{}:/sandbox/output",
                convert_path_in_container_to_path_in_host(&tests_paths.input)?.display()
            ),
            "-v",
            &format!(
                "{}:/sandbox/final_output:ro",
                convert_path_in_container_to_path_in_host(final_output)?.display()
            ),
            "-v",
            &format!(
                "{}:/sandbox/answer:ro",
                convert_path_in_container_to_path_in_host(answer_path)?.display()
            ),
            "-w",
            "/sandbox",
            &sandbox_image,
            "timeout",
            "-s",
            "KILL",
            &format!("{}s", f64::from(config.time_limit_ms) / 1000.),
            "/sandbox/bin",
            "/sandbox/input",
            "/sandbox/output",
            "/sandbox/final_output",
            "/sandbox/answer",
            &stage.to_string(),
        ])
        .status();

    let checker_status = checker_cmd.await;
    checker_status.map_or(Err(TestingVerdict::Fail), |status| match status.code() {
        Some(CHECKER_OK) => Ok(Verdict::Ok),
        Some(CHECKER_WA) => Ok(Verdict::WrongAnswer),
        Some(CHECKER_PE) => Ok(Verdict::PresentationError),
        _ => Err(TestingVerdict::Fail),
    })
}
