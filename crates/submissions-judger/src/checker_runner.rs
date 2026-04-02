use crate::{
    constants::{CHECKER_OK, CHECKER_PE, CHECKER_WA},
    tools::convert_path_in_container_to_path_in_host,
};
use models::{
    problem_config::ProblemConfig,
    testing::{CheckerResult, TestsPaths},
    verdicts::{SubgroupVerdict, TotalVerdict},
};
use std::{
    env,
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};
use tokio::{
    fs::{self, File},
    process::Command,
    time::timeout,
};
use tools::map::MapLogExt;

#[allow(clippy::cast_sign_loss)]
pub async fn get_checker_result(
    config: &ProblemConfig,
    input_path: &Path,
    answer_path: PathBuf,
    tests_paths: &TestsPaths,
) -> Result<CheckerResult, TotalVerdict> {
    log::info!("Open stderr file");

    let stderr_file = File::create(tests_paths.error.clone())
        .await
        .map_log(TotalVerdict::InvalidProblem)?;

    log::info!("Run checker cmd");
    let sandbox_image = env::var("SANDBOX_IMAGE").map_log(TotalVerdict::Bug)?;

    let mut checker_cmd = Command::new("docker")
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
            "/sandbox/bin",
            "/sandbox/input",
            "/sandbox/output",
            "/sandbox/answer",
        ])
        .stderr(Stdio::from(stderr_file.into_std().await))
        .spawn()
        .map_log(TotalVerdict::Bug)?;

    let timeout_duration = Duration::from_millis(config.time_limit_ms as u64);
    let checker_status = timeout(timeout_duration, checker_cmd.wait()).await;
    _ = checker_cmd.kill();
    let checker_status = checker_status.map_log(TotalVerdict::InvalidProblem)?;

    log::info!("Check checker status");
    match checker_status {
        Err(_) => Err(TotalVerdict::InvalidProblem),
        Ok(status) => {
            let checker_msg = fs::read_to_string(tests_paths.error.clone())
                .await
                .map_log(TotalVerdict::InvalidProblem)?;

            match status.code() {
                Some(CHECKER_OK) => Ok(CheckerResult {
                    verdict: SubgroupVerdict::Ok,
                    checker_msg,
                }),
                Some(CHECKER_WA) => Ok(CheckerResult {
                    verdict: SubgroupVerdict::WrongAnswer,
                    checker_msg,
                }),
                Some(CHECKER_PE) => Ok(CheckerResult {
                    verdict: SubgroupVerdict::PresentationError,
                    checker_msg,
                }),
                _ => Err(TotalVerdict::InvalidProblem),
            }
        }
    }
}
