use crate::constants::{VERDICT_MLE, VERDICT_OK, VERDICT_TLE};
use aj_models::{
    problems::ProblemConfig,
    verdicts::{TestingVerdict, Verdict},
};
use models::testing::TestsPaths;
use std::{env, path::Path, process::Stdio};
use tokio::{fs::File, process::Command};
use tools::host::ToHostExt;

pub async fn get_solution_verdict(
    config: &ProblemConfig,
    input_path: &Path,
    tests_paths: &TestsPaths,
) -> Result<Verdict, TestingVerdict> {
    let stdin_file = File::open(input_path).await.map_err(|error| {
        tracing::error!(%error, path = %input_path.display(), "failed to open test input");
        TestingVerdict::Fail
    })?;
    let stdout_file = File::create(tests_paths.output.clone())
        .await
        .map_err(|error| {
            tracing::error!(%error, path = %tests_paths.output.display(), "failed to create test output");
            TestingVerdict::Fail
        })?;

    let sandbox_image = env::var("SANDBOX_IMAGE").map_err(|error| {
        tracing::error!(%error, "SANDBOX_IMAGE is not set");
        TestingVerdict::Fail
    })?;

    let solution_cmd = Command::new("docker")
        .args([
            "run",
            "--rm",
            "--init",
            "--network",
            "none",
            "--memory",
            &format!("{}m", config.memory_limit_mb * 2),
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
            "--tmpfs",
            &format!(
                "/tmp:rw,exec,size={}m,mode=1777",
                config.memory_limit_mb * 2,
            ),
            "-v",
            &format!(
                "{}:/sandbox/bin:ro",
                tests_paths.solution.to_host()?.display()
            ),
            "-w",
            "/sandbox",
            &sandbox_image,
            "timeout",
            "-s",
            "KILL",
            &format!("{}s", f64::from(config.time_limit_ms) * 2. / 1000.),
            "/sandbox/bin",
        ])
        .stdin(Stdio::from(stdin_file.into_std().await))
        .stdout(Stdio::from(stdout_file.into_std().await))
        .status();

    let solution_status = solution_cmd.await;

    solution_status.map_or_else(
        |error| {
            tracing::error!(%error, "failed to run solution sandbox");
            Ok(Verdict::TimeLimitExceeded)
        },
        |status| match status.code() {
            Some(VERDICT_OK) => Ok(Verdict::Ok),
            Some(VERDICT_MLE) => Ok(Verdict::MemoryLimitExceeded),
            Some(VERDICT_TLE) | None => Ok(Verdict::TimeLimitExceeded),
            Some(code) => {
                tracing::debug!(code, "solution exited with non-verdict code");
                Ok(Verdict::RuntimeError)
            }
        },
    )
}
