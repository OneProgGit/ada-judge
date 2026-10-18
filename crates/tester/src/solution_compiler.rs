use std::{
    env,
    path::{Path, PathBuf},
    process::Stdio,
};

use aj_models::{testing::Language, verdicts::TestingVerdict};
use models::testing::{SubmissionTask, TestsPaths};
use tokio::process::Command;
use tools::host::ToHostExt;

pub async fn compile_solution(
    run_path: &Path,
    tests_paths: &TestsPaths,
    submission: &SubmissionTask,
) -> Result<(), TestingVerdict> {
    let sandbox_image = env::var("SANDBOX_IMAGE").map_err(|_| TestingVerdict::Fail)?;

    let compile_cmd = match submission.language {
        Language::C => "clang",
        Language::Cpp => "clang++",
        Language::Go => "go",
        Language::Rust => "rustc",
        Language::Python => "pyinstaller",
        Language::FreePascal => "fpc",
        Language::Unknown => return Err(TestingVerdict::Fail),
    };
    let solution_source_path = PathBuf::from("env")
        .join(
            tests_paths
                .solution_source
                .file_name()
                .ok_or(TestingVerdict::Fail)?
                .to_string_lossy()
                .to_string(),
        )
        .to_string_lossy()
        .to_string();
    let solution_path = PathBuf::from("env")
        .join(
            tests_paths
                .solution
                .file_name()
                .ok_or(TestingVerdict::Fail)?
                .to_string_lossy()
                .to_string(),
        )
        .to_string_lossy()
        .to_string();

    let compilation_result = Command::new("docker")
        .args([
            "run",
            "--rm",
            "--init",
            "--network",
            "none",
            "--cpus",
            "0.5",
            "--pids-limit",
            "128",
            "--cap-drop",
            "ALL",
            "-i",
            "--security-opt",
            "no-new-privileges",
            "-v",
            &format!("{}:/sandbox/env", run_path.to_host()?.display()),
            "-w",
            "/sandbox",
            &sandbox_image,
        ])
        .args(match submission.language {
            Language::C | Language::Cpp => vec![
                compile_cmd,
                "-O2",
                "-pipe",
                "-flto",
                "-s",
                &solution_source_path,
                "-o",
                &solution_path,
            ],
            Language::Go => vec![
                compile_cmd,
                "build",
                "-o",
                &solution_path,
                &solution_source_path,
            ],
            Language::Rust => vec![
                compile_cmd,
                &solution_source_path,
                "-O",
                "-C",
                "lto",
                "-o",
                &solution_path,
            ],
            Language::Python => vec![
                compile_cmd,
                "--onefile",
                "-n",
                "run",
                "--distpath",
                "/sandbox/env",
                "--workpath",
                "/sandbox/build",
                "--specpath",
                "/sandbox/build",
                &solution_source_path,
            ],
            Language::FreePascal => vec![compile_cmd, "-O2", &solution_source_path],
            Language::Unknown => unreachable!(),
        })
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|error| {
            tracing::error!(%error, "failed to run compilation sandbox");
            TestingVerdict::CompilationError
        })?;
    match compilation_result.status.code() {
        Some(0) => Ok(()),
        code => {
            tracing::warn!(
                ?code,
                stderr = %String::from_utf8_lossy(&compilation_result.stderr),
                "solution compilation failed"
            );
            Err(TestingVerdict::CompilationError)
        }
    }
}
