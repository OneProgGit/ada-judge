use std::{
    env,
    path::{Path, PathBuf},
};

use aj_models::{
    errors::{AdaJudgeError, InvalidProblem},
    testing::Language,
};
use tokio::process::Command;
use tools::host::ToHostExt;

#[allow(clippy::too_many_lines)]
pub async fn compile_checker(
    checker_path: &Path,
    checker_lang: &Language,
) -> Result<(), AdaJudgeError> {
    let sandbox_image = env::var("SANDBOX_IMAGE").map_err(|_| AdaJudgeError::Internal)?;

    let compile_cmd = match checker_lang {
        Language::C => "clang",
        Language::Cpp => "clang++",
        Language::Go => "go",
        Language::Rust => "rustc",
        Language::Python => "pyinstaller",
        Language::FreePascal => "fpc",
        Language::Unknown => return Err(AdaJudgeError::BadRequest),
    };
    let checker_source_path = PathBuf::from("env")
        .join(checker_path)
        .to_string_lossy()
        .to_string();
    let checker_path = PathBuf::from("env")
        .join("checker")
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
                checker_path
                    .to_host()
                    .map_err(|_| AdaJudgeError::Internal)?
                    .display()
            ),
            "-w",
            "/sandbox",
            &sandbox_image,
        ])
        .args(match checker_lang {
            Language::C | Language::Cpp => vec![
                compile_cmd,
                "-O2",
                "-pipe",
                "-flto",
                "-s",
                &checker_source_path,
                "-o",
                &checker_path,
            ],
            Language::Go => vec![
                compile_cmd,
                "build",
                "-o",
                &checker_path,
                &checker_source_path,
            ],
            Language::Rust => vec![
                compile_cmd,
                &checker_source_path,
                "-O",
                "-C",
                "lto",
                "-o",
                &checker_path,
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
                &checker_source_path,
            ],
            Language::FreePascal => vec![compile_cmd, "-O2", &checker_source_path],
            Language::Unknown => unreachable!(),
        })
        .spawn()
        .map_err(|_| AdaJudgeError::InvalidProblem(InvalidProblem::CheckerCompilationError))?;
    let compilation_result = compile_cmd
        .wait()
        .await
        .map_err(|_| AdaJudgeError::InvalidProblem(InvalidProblem::CheckerCompilationError))?;

    _ = compile_cmd.kill();
    match compilation_result.code() {
        Some(0) => Ok(()),
        _ => Err(AdaJudgeError::InvalidProblem(
            InvalidProblem::CheckerCompilationError,
        )),
    }
}
