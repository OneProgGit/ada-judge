use crate::problem_config::ProblemConfig;
use fs_extra::dir::CopyOptions;
use models::{testing::TestsPaths, verdicts::TotalVerdict};
use std::path::PathBuf;
use tokio::fs;
use tools::map::MapLogExt;

pub async fn prepare_test_env(
    problem_path: PathBuf,
    config: &ProblemConfig,
    tests_paths: &TestsPaths,
) -> Result<(), TotalVerdict> {
    log::info!("Copy checker");
    fs::copy(
        problem_path.join(config.checker.path.clone()),
        tests_paths.checker.clone(),
    )
    .await
    .map_log(TotalVerdict::InvalidProblem)?;

    let mut opt = CopyOptions::new();
    opt.overwrite = true;
    opt.copy_inside = true;
    opt.content_only = false;
    opt.skip_exist = false;

    log::info!("Copy tests");

    let from_tests_dir = problem_path.join(config.tests.path.clone());
    let to_tests_dir = tests_paths.tests.clone();
    tokio::task::spawn_blocking(move || fs_extra::dir::copy(from_tests_dir, to_tests_dir, &opt))
        .await
        .map_log(TotalVerdict::Bug)?
        .map_log(TotalVerdict::InvalidProblem)?;

    log::info!("Create stderr file");
    fs::write(tests_paths.error.clone(), "")
        .await
        .map_log(TotalVerdict::InvalidProblem)?;

    Ok(())
}
