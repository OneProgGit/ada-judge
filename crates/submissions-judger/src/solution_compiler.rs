use std::{
    env,
    path::{Path, PathBuf},
};

use aj_models::{testing::Language, verdicts::TotalVerdict};
use models::testing::{SubmissionTask, TestsPaths};
use tokio::process::Command;
use tools::map::MapLogExt;

use crate::tools::convert_path_in_container_to_path_in_host;

pub async fn compile_solution(
    run_path: &Path,
    tests_paths: &TestsPaths,
    submission: &SubmissionTask,
) -> Result<(), TotalVerdict> {
    let sandbox_image = env::var("SANDBOX_IMAGE").map_log(TotalVerdict::Bug)?;

    let compile_cmd = match submission.language {
        Language::Clang => "clang",
        Language::Clangpp => "clang++",
        Language::Go => "go",
        Language::Rust => "rustc",
        Language::Python => "pyinstaller",
        Language::FreePascal => "fpc",
        Language::Unknown => return Err(TotalVerdict::InvalidRequest),
    };
    let solution_source_path = PathBuf::from("env")
        .join(
            tests_paths
                .solution_source
                .file_name()
                .ok_or(TotalVerdict::Bug)?
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
                .ok_or(TotalVerdict::Bug)?
                .to_string_lossy()
                .to_string(),
        )
        .to_string_lossy()
        .to_string();

    let mut compile_cmd = Command::new("docker")
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
            &format!(
                "{}:/sandbox/env",
                convert_path_in_container_to_path_in_host(run_path)?.display(),
            ),
            "-w",
            "/sandbox",
            &sandbox_image,
        ])
        .args(match submission.language {
            Language::Clang | Language::Clangpp => vec![
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
        .spawn()
        .map_log(TotalVerdict::CompilationError)?;
    let compilation_result = compile_cmd
        .wait()
        .await
        .map_log(TotalVerdict::CompilationError)?;

    _ = compile_cmd.kill();
    match compilation_result.code() {
        Some(0) => {
            log::info!("Solution compiled successfully");
            Ok(())
        }
        Some(_) => {
            log::error!("Compilation status is not zero");
            Err(TotalVerdict::CompilationError)
        }
        None => {
            log::error!("No compilation status");
            Err(TotalVerdict::CompilationError)
        }
    }
}
