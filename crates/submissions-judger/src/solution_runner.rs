use crate::{
    constants::{VERDICT_MLE, VERDICT_OK, VERDICT_TLE},
    tools::convert_path_in_container_to_path_in_host,
};
use ada_judge_public_models::{
    problems::ProblemConfig,
    verdicts::{SubgroupVerdict, TotalVerdict},
};
use models::testing::TestsPaths;
use std::{env, path::Path, process::Stdio};
use tokio::{fs::File, process::Command};
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
    let sandbox_image = env::var("SANDBOX_IMAGE").map_log(TotalVerdict::Bug)?;

    let solution_cmd = Command::new("docker")
        .args([
            "run",
            "--rm",
            "--init",
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
            &sandbox_image,
            "timeout",
            &format!("{}s", f64::from(config.time_limit_ms) / 1000.),
            "/sandbox/bin",
        ])
        .stdin(Stdio::from(stdin_file.into_std().await))
        .stdout(Stdio::from(stdout_file.into_std().await))
        .stderr(Stdio::from(stderr_file.into_std().await))
        .status();

    let solution_status = solution_cmd.await;

    log::info!("Check solution status");
    solution_status.map_or(
        Ok(SubgroupVerdict::TimeLimitExceeded),
        |status| match status.code() {
            Some(VERDICT_OK) => Ok(SubgroupVerdict::Ok),
            Some(VERDICT_MLE) => Ok(SubgroupVerdict::MemoryLimitExceeded),
            Some(VERDICT_TLE) | None => Ok(SubgroupVerdict::TimeLimitExceeded),
            Some(_code) => Ok(SubgroupVerdict::RuntimeError),
        },
    )
}
