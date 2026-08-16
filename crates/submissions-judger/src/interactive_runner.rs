use crate::{
    constants::{CHECKER_OK, CHECKER_PE, CHECKER_WA, VERDICT_MLE, VERDICT_OK, VERDICT_TLE},
    tools::container_to_host,
};
use aj_models::{
    problems::ProblemConfig,
    verdicts::{TestingVerdict, Verdict},
};
use models::testing::TestsPaths;
use std::{
    env,
    path::{Path, PathBuf},
    process::Stdio,
};
use tokio::{
    fs::{self, File},
    process::Command,
};
use tools::map::MapLogExt;

#[allow(clippy::cast_sign_loss)]
pub async fn get_interactive_verdict(
    config: &ProblemConfig,
    input_path: &Path,
    output_path: &Path,
    answer_path: &Path,
) -> Result<Verdict, TestingVerdict> {
    fs::create_dir_all(&tests_paths.fifo)
        .await
        .map_log(TestingVerdict::Bug)?;

    let checker_to_solution = format!("{}/checker_to_solution", tests_paths.fifo.display());
    let solution_to_checker = format!("{}/solution_to_checker", tests_paths.fifo.display());

    let checker_to_solution_path = PathBuf::from(checker_to_solution.clone());
    let solution_to_checker_path = PathBuf::from(solution_to_checker.clone());

    for path in [&checker_to_solution, &solution_to_checker] {
        if PathBuf::from(path).exists() {
            fs::remove_file(path).await.map_log(TestingVerdict::Bug)?;
        }

        let status = Command::new("mkfifo")
            .arg(path)
            .status()
            .await
            .map_log(TestingVerdict::Bug)?;
        if !status.success() {
            return Err(TestingVerdict::Bug);
        }
    }

    let sandbox_image = env::var("SANDBOX_IMAGE").map_log(TestingVerdict::Bug)?;

    let mut checker_child = Command::new("docker")
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
                container_to_host(&tests_paths.checker)?.display()
            ),
            "-v",
            &format!(
                "{}:/sandbox/input:ro",
                container_to_host(&solution_to_checker_path)?.display()
            ),
            "-v",
            &format!(
                "{}:/sandbox/output",
                container_to_host(&checker_to_solution_path)?.display()
            ),
            "-v",
            &format!(
                "{}:/sandbox/answer:ro",
                container_to_host(answer_path)?.display()
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
        .spawn()
        .map_log(TestingVerdict::Bug)?;

    let (checker_to_solution_read, solution_to_checker_write) = tokio::join!(
        File::open(&checker_to_solution_path),
        File::create(&solution_to_checker_path)
    );
    let checker_to_solution_read = checker_to_solution_read
        .map_log(TestingVerdict::Bug)?
        .into_std()
        .await;
    let solution_to_checker_write = solution_to_checker_write
        .map_log(TestingVerdict::Bug)?
        .into_std()
        .await;

    let mut solution_child = Command::new("docker")
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
                container_to_host(&tests_paths.solution)?.display()
            ),
            "-w",
            "/sandbox",
            &sandbox_image,
            "timeout",
            "-s",
            "KILL",
            &format!("{}s", f64::from(config.time_limit_ms) / 1000.),
            "/sandbox/bin",
        ])
        .stdin(Stdio::from(checker_to_solution_read))
        .stdout(Stdio::from(solution_to_checker_write))
        .spawn()
        .map_log(TestingVerdict::Bug)?;

    let (checker_status, solution_status) =
        tokio::join!(checker_child.wait(), solution_child.wait());
    solution_status.map_or(Ok(Verdict::TimeLimitExceeded), |status| {
        match status.code() {
            Some(VERDICT_OK) => {
                checker_status.map_or(Err(TestingVerdict::InvalidProblem), |status| {
                    match status.code() {
                        Some(CHECKER_OK) => Ok(Verdict::Ok),
                        Some(CHECKER_WA) => Ok(Verdict::WrongAnswer),
                        Some(CHECKER_PE) => Ok(Verdict::PresentationError),
                        _ => Err(TestingVerdict::InvalidProblem),
                    }
                })
            }
            Some(VERDICT_MLE) => Ok(Verdict::MemoryLimitExceeded),
            Some(VERDICT_TLE) | None => Ok(Verdict::TimeLimitExceeded),
            Some(_code) => Ok(Verdict::RuntimeError),
        }
    })
}

#[allow(clippy::cast_sign_loss)]
pub async fn get_interactive_run_twice_verdict(
    config: &ProblemConfig,
    input_path: &Path,
    output_path: &Path,
    final_output: &Path,
    answer_path: &Path,
    stage: i32,
) -> Result<Verdict, TestingVerdict> {
    fs::create_dir_all(&tests_paths.fifo)
        .await
        .map_log(TestingVerdict::Bug)?;

    let checker_to_solution = format!("{}/checker_to_solution", tests_paths.fifo.display());
    let solution_to_checker = format!("{}/solution_to_checker", tests_paths.fifo.display());

    let checker_to_solution_path = PathBuf::from(checker_to_solution.clone());
    let solution_to_checker_path = PathBuf::from(solution_to_checker.clone());

    for path in [&checker_to_solution, &solution_to_checker] {
        if PathBuf::from(path).exists() {
            fs::remove_file(path).await.map_log(TestingVerdict::Bug)?;
        }

        let status = Command::new("mkfifo")
            .arg(path)
            .status()
            .await
            .map_log(TestingVerdict::Bug)?;
        if !status.success() {
            return Err(TestingVerdict::Bug);
        }
    }

    let sandbox_image = env::var("SANDBOX_IMAGE").map_log(TestingVerdict::Bug)?;

    let mut checker_child = Command::new("docker")
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
                container_to_host(&tests_paths.checker)?.display()
            ),
            "-v",
            &format!(
                "{}:/sandbox/input:ro",
                container_to_host(&solution_to_checker_path)?.display()
            ),
            "-v",
            &format!(
                "{}:/sandbox/output",
                container_to_host(&checker_to_solution_path)?.display()
            ),
            "-v",
            &format!(
                "{}:/sandbox/final_output:ro",
                container_to_host(final_output)?.display()
            ),
            "-v",
            &format!(
                "{}:/sandbox/answer:ro",
                container_to_host(answer_path)?.display()
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
        .spawn()
        .map_log(TestingVerdict::Bug)?;

    let (checker_to_solution_read, solution_to_checker_write) = tokio::join!(
        File::open(&checker_to_solution_path),
        File::create(&solution_to_checker_path)
    );
    let checker_to_solution_read = checker_to_solution_read
        .map_log(TestingVerdict::Bug)?
        .into_std()
        .await;
    let solution_to_checker_write = solution_to_checker_write
        .map_log(TestingVerdict::Bug)?
        .into_std()
        .await;

    let mut solution_child = Command::new("docker")
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
                container_to_host(&tests_paths.solution)?.display()
            ),
            "-w",
            "/sandbox",
            &sandbox_image,
            "timeout",
            "-s",
            "KILL",
            &format!("{}s", f64::from(config.time_limit_ms) / 1000.),
            "/sandbox/bin",
        ])
        .stdin(Stdio::from(checker_to_solution_read))
        .stdout(Stdio::from(solution_to_checker_write))
        .spawn()
        .map_log(TestingVerdict::Bug)?;

    let (checker_status, solution_status) =
        tokio::join!(checker_child.wait(), solution_child.wait());
    solution_status.map_or(Ok(Verdict::TimeLimitExceeded), |status| {
        match status.code() {
            Some(VERDICT_OK) => {
                checker_status.map_or(Err(TestingVerdict::InvalidProblem), |status| {
                    match status.code() {
                        Some(CHECKER_OK) => Ok(Verdict::Ok),
                        Some(CHECKER_WA) => Ok(Verdict::WrongAnswer),
                        Some(CHECKER_PE) => Ok(Verdict::PresentationError),
                        _ => Err(TestingVerdict::InvalidProblem),
                    }
                })
            }
            Some(VERDICT_MLE) => Ok(Verdict::MemoryLimitExceeded),
            Some(VERDICT_TLE) | None => Ok(Verdict::TimeLimitExceeded),
            Some(_code) => Ok(Verdict::RuntimeError),
        }
    })
}
