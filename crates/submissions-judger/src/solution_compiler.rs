use models::{
    testing::{Language, SubmissionTask, TestsPaths},
    verdicts::TotalVerdict,
};
use tokio::process::Command;
use tools::map::MapLogExt;

pub async fn compile_solution(
    tests_paths: &TestsPaths,
    submission: &SubmissionTask,
) -> Result<(), TotalVerdict> {
    let mut compile_cmd = Command::new(match submission.lang {
        Language::Clang => "clang++",
        Language::Go => "go",
        Language::Rust => "rustc",
        Language::Haskell => "ghc",
    });
    let compile_cmd = match submission.lang {
        Language::Clang => compile_cmd
            .args(["-O2", "-pipe", "-march=native", "-flto", "-s"])
            .arg(&tests_paths.solution_source)
            .arg("-o")
            .arg(&tests_paths.solution),
        Language::Go => compile_cmd
            .args(["build", "-o"])
            .args([&tests_paths.solution, &tests_paths.solution_source]),
        Language::Rust => compile_cmd
            .arg(&tests_paths.solution_source)
            .args(["-O", "-C", "target-cpu=native", "-C", "lto", "-o"])
            .arg(&tests_paths.solution),
        Language::Haskell => compile_cmd.arg("-O2").arg(&tests_paths.solution),
    };
    let mut compile_cmd = compile_cmd
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
