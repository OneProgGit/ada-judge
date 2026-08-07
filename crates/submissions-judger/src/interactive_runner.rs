use crate::{
    constants::{CHECKER_OK, CHECKER_PE, CHECKER_WA, VERDICT_MLE, VERDICT_OK, VERDICT_TLE},
    tools::convert_path_in_container_to_path_in_host,
};
use aj_models::{
    problems::ProblemConfig,
    verdicts::{SubgroupVerdict, TotalVerdict},
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
pub async fn get_run_interactive_verdict(
    config: &ProblemConfig,
    answer_path: &Path,
    tests_paths: &TestsPaths,
) -> Result<SubgroupVerdict, TotalVerdict> {
    fs::create_dir_all(&tests_paths.fifo)
        .await
        .map_log(TotalVerdict::Bug)?;

    let checker_to_solution = format!("{}/checker_to_solution", tests_paths.fifo.display());
    let solution_to_checker = format!("{}/solution_to_checker", tests_paths.fifo.display());

    let checker_to_solution_path = PathBuf::from(checker_to_solution.clone());
    let solution_to_checker_path = PathBuf::from(solution_to_checker.clone());

    for path in [&checker_to_solution, &solution_to_checker] {
        if PathBuf::from(path).exists() {
            fs::remove_file(path).await.map_log(TotalVerdict::Bug)?;
        }

        let status = Command::new("mkfifo")
            .arg(path)
            .status()
            .await
            .map_log(TotalVerdict::Bug)?;
        if !status.success() {
            return Err(TotalVerdict::Bug);
        }
    }

    let sandbox_image = env::var("SANDBOX_IMAGE").map_log(TotalVerdict::Bug)?;

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
                convert_path_in_container_to_path_in_host(&tests_paths.checker)?.display()
            ),
            "-v",
            &format!(
                "{}:/sandbox/input:ro",
                convert_path_in_container_to_path_in_host(&solution_to_checker_path)?.display()
            ),
            "-v",
            &format!(
                "{}:/sandbox/output",
                convert_path_in_container_to_path_in_host(&checker_to_solution_path)?.display()
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
        ])
        .spawn()
        .map_log(TotalVerdict::Bug)?;

    let (checker_to_solution_read, solution_to_checker_write) = tokio::join!(
        File::open(&checker_to_solution_path),
        File::create(&solution_to_checker_path)
    );
    let checker_to_solution_read = checker_to_solution_read
        .map_log(TotalVerdict::Bug)?
        .into_std()
        .await;
    let solution_to_checker_write = solution_to_checker_write
        .map_log(TotalVerdict::Bug)?
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
                convert_path_in_container_to_path_in_host(&tests_paths.solution)?.display()
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
        .map_log(TotalVerdict::Bug)?;

    let (checker_status, solution_status) =
        tokio::join!(checker_child.wait(), solution_child.wait());

    log::info!("Check solution status");
    solution_status.map_or(
        Ok(SubgroupVerdict::TimeLimitExceeded),
        |status| match status.code() {
            Some(VERDICT_OK) => {
                checker_status.map_or(Err(TotalVerdict::InvalidProblem), |status| {
                    match status.code() {
                        Some(CHECKER_OK) => Ok(SubgroupVerdict::Ok),
                        Some(CHECKER_WA) => Ok(SubgroupVerdict::WrongAnswer),
                        Some(CHECKER_PE) => Ok(SubgroupVerdict::PresentationError),
                        _ => Err(TotalVerdict::InvalidProblem),
                    }
                })
            }
            Some(VERDICT_MLE) => Ok(SubgroupVerdict::MemoryLimitExceeded),
            Some(VERDICT_TLE) | None => Ok(SubgroupVerdict::TimeLimitExceeded),
            Some(_code) => Ok(SubgroupVerdict::RuntimeError),
        },
    )
}

#[allow(clippy::cast_sign_loss)]
pub async fn get_run_interactive_verdict_run_twice(
    config: &ProblemConfig,
    answer_path: &Path,
    final_output: &Path,
    tests_paths: &TestsPaths,
    stage: i32,
) -> Result<SubgroupVerdict, TotalVerdict> {
    fs::create_dir_all(&tests_paths.fifo)
        .await
        .map_log(TotalVerdict::Bug)?;

    let checker_to_solution = format!("{}/checker_to_solution", tests_paths.fifo.display());
    let solution_to_checker = format!("{}/solution_to_checker", tests_paths.fifo.display());

    let checker_to_solution_path = PathBuf::from(checker_to_solution.clone());
    let solution_to_checker_path = PathBuf::from(solution_to_checker.clone());

    for path in [&checker_to_solution, &solution_to_checker] {
        if PathBuf::from(path).exists() {
            fs::remove_file(path).await.map_log(TotalVerdict::Bug)?;
        }

        let status = Command::new("mkfifo")
            .arg(path)
            .status()
            .await
            .map_log(TotalVerdict::Bug)?;
        if !status.success() {
            return Err(TotalVerdict::Bug);
        }
    }

    let sandbox_image = env::var("SANDBOX_IMAGE").map_log(TotalVerdict::Bug)?;

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
                convert_path_in_container_to_path_in_host(&tests_paths.checker)?.display()
            ),
            "-v",
            &format!(
                "{}:/sandbox/input:ro",
                convert_path_in_container_to_path_in_host(&solution_to_checker_path)?.display()
            ),
            "-v",
            &format!(
                "{}:/sandbox/output",
                convert_path_in_container_to_path_in_host(&checker_to_solution_path)?.display()
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
        .spawn()
        .map_log(TotalVerdict::Bug)?;

    let (checker_to_solution_read, solution_to_checker_write) = tokio::join!(
        File::open(&checker_to_solution_path),
        File::create(&solution_to_checker_path)
    );
    let checker_to_solution_read = checker_to_solution_read
        .map_log(TotalVerdict::Bug)?
        .into_std()
        .await;
    let solution_to_checker_write = solution_to_checker_write
        .map_log(TotalVerdict::Bug)?
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
                convert_path_in_container_to_path_in_host(&tests_paths.solution)?.display()
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
        .map_log(TotalVerdict::Bug)?;

    let (checker_status, solution_status) =
        tokio::join!(checker_child.wait(), solution_child.wait());

    log::info!("Check solution status");
    solution_status.map_or(
        Ok(SubgroupVerdict::TimeLimitExceeded),
        |status| match status.code() {
            Some(VERDICT_OK) => {
                checker_status.map_or(Err(TotalVerdict::InvalidProblem), |status| {
                    match status.code() {
                        Some(CHECKER_OK) => Ok(SubgroupVerdict::Ok),
                        Some(CHECKER_WA) => Ok(SubgroupVerdict::WrongAnswer),
                        Some(CHECKER_PE) => Ok(SubgroupVerdict::PresentationError),
                        _ => Err(TotalVerdict::InvalidProblem),
                    }
                })
            }
            Some(VERDICT_MLE) => Ok(SubgroupVerdict::MemoryLimitExceeded),
            Some(VERDICT_TLE) | None => Ok(SubgroupVerdict::TimeLimitExceeded),
            Some(_code) => Ok(SubgroupVerdict::RuntimeError),
        },
    )
}
