use crate::{
    constants::{VERDICT_MLE, VERDICT_OK},
    tools::convert_path_in_container_to_path_in_host,
};
use models::{
    problem_config::ProblemConfig,
    testing::TestsPaths,
    verdicts::{SubgroupVerdict, TotalVerdict},
};
use std::{path::Path, process::Stdio, time::Duration};
use tokio::{fs::File, process::Command, time::timeout};
use tools::map::MapLogExt;

#[allow(clippy::cast_sign_loss)]
pub async fn get_run_solution_verdict(
    config: &ProblemConfig,
    input_path: &Path,
    tests_paths: &TestsPaths,
) -> Result<SubgroupVerdict, TotalVerdict> {
    log::info!("Open stdin file");
    let stdin_file = File::open(input_path)
        .await
        .map_log(TotalVerdict::InvalidProblem)?;
    log::info!("Open stdout file");
    let stdout_file = File::create(tests_paths.output.clone())
        .await
        .map_log(TotalVerdict::InvalidProblem)?;
    log::info!("Open stderr file");
    let stderr_file = File::create(tests_paths.error.clone())
        .await
        .map_log(TotalVerdict::InvalidProblem)?;

    log::info!("Run solution cmd");
    let mut solution_cmd = Command::new("docker")
        .args([
            "run",
            "--rm",
            "--network",
            "none",
            "--memory",
            &format!("{}m", config.memory_limit_mb),
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
            "sandbox-runner",
            "/sandbox/bin",
        ])
        .stdin(Stdio::from(stdin_file.into_std().await))
        .stdout(Stdio::from(stdout_file.into_std().await))
        .stderr(Stdio::from(stderr_file.into_std().await))
        .spawn()
        .map_log(TotalVerdict::Bug)?;

    let timeout_duration = Duration::from_millis(config.time_limit_ms as u64);
    let solution_status = timeout(timeout_duration, solution_cmd.wait()).await;
    _ = solution_cmd.kill();
    let solution_status = match solution_status {
        Ok(solution_status) => solution_status,
        Err(e) => {
            log::error!("{e}");
            return Ok(SubgroupVerdict::TimeLimitExceeded);
        }
    };
    log::info!("Check solution status");
    solution_status.map_or(
        Ok(SubgroupVerdict::TimeLimitExceeded),
        |status| match status.code() {
            Some(VERDICT_OK) => Ok(SubgroupVerdict::Ok),
            Some(VERDICT_MLE) => Ok(SubgroupVerdict::MemoryLimitExceeded),
            _ => Ok(SubgroupVerdict::RuntimeError),
        },
    )
}
