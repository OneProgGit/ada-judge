//! Submissions judger worker for `ada-judge`

#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![deny(warnings)]
#![deny(missing_docs)]
#![deny(rustdoc::all)]
#![deny(rustdoc::broken_intra_doc_links)]
#![forbid(unsafe_code)]

use ::tools::map::MapLogExt;
use ada_judge_public_models::{
    problems::{ProblemConfig, Subgroup},
    testing::SubgroupResult,
    verdicts::{SubgroupVerdict, TotalVerdict},
};
use apalis::prelude::{BoxDynError, Data};
use checker_runner::get_checker_result;
use database::get_problem_by_id;
use database::tools::MapDbExt;
use models::testing::{SubmissionTask, TestsPaths};
use solution_compiler::compile_solution;
use solution_runner::get_run_solution_verdict;
use sqlx::PgPool;
use std::path::Path;
use test_env_preparer::prepare_test_env;
use tokio::fs::read_to_string;

mod checker_runner;
mod constants;
mod solution_compiler;
mod solution_runner;
mod test_env_preparer;
mod tools;

async fn get_single_test_verdict(
    config: &ProblemConfig,
    tests_paths: &TestsPaths,
    test_id: i32,
) -> Result<SubgroupVerdict, TotalVerdict> {
    let test_path = tests_paths.tests.join(test_id.to_string());

    let input_path = test_path.join("in");
    let answer_path = test_path.join("out");

    log::info!("Run solution");
    let solution_verdict = get_run_solution_verdict(config, &input_path, tests_paths).await?;

    if solution_verdict != SubgroupVerdict::Ok {
        log::error!("Run result isn't OK");
        return Ok(solution_verdict);
    }

    log::info!("Run checker");
    get_checker_result(config, &input_path, answer_path, tests_paths).await
}

async fn write_subgroup_result(
    test_result: &mut SubgroupResult,
    subgroup: &Subgroup,
    config: &ProblemConfig,
    tests_paths: &TestsPaths,
) -> Result<(), TotalVerdict> {
    for test_id in &subgroup.tests {
        let test_id = *test_id;
        log::info!("Run test #{test_id}");

        test_result.test = test_id;
        let subgroup_verdict = get_single_test_verdict(config, tests_paths, test_id).await?;

        test_result.subgroup_verdict = subgroup_verdict;
        test_result.test = test_id;

        if test_result.subgroup_verdict != SubgroupVerdict::Ok {
            log::error!(
                "Verdict {} isn't OK, skip testing",
                test_result.subgroup_verdict
            );
            return Ok(());
        }
    }
    test_result.score = subgroup.score;
    Ok(())
}

/// TODO: move it to api
#[allow(dead_code)]
async fn load_config(problem_path: &Path) -> Result<ProblemConfig, TotalVerdict> {
    let config_text = read_to_string(problem_path.join("config.toml"))
        .await
        .map_log(TotalVerdict::InvalidProblem)?;

    toml::from_str::<ProblemConfig>(&config_text).map_log(TotalVerdict::InvalidProblem)
}

fn assert_subgroups_correctness(config: &ProblemConfig) -> Result<(), TotalVerdict> {
    for (i, subgroup) in config.subgroups.iter().enumerate() {
        log::info!("Check subgroup #{i} for correctness");
        for x in &subgroup.depends_on {
            if *x >= i {
                log::error!("Subgroup depends on a subgroup that has index less than it's");
                return Err(TotalVerdict::InvalidProblem);
            }
        }
    }
    Ok(())
}

fn does_subgroup_need_to_be_tested_on(
    subgroup: &Subgroup,
    subgroups_results: &[SubgroupResult],
) -> bool {
    for i in &subgroup.depends_on {
        if subgroups_results[*i].subgroup_verdict != SubgroupVerdict::Ok {
            return false;
        }
    }
    true
}

/// Tests submission for a problem on all test subgroups and writes a total verdict and a verdict for each subgroup
/// # Errors
/// Returns an error if:
/// - Request is invalid
/// - Problem is invalid
/// - Verdict isn't Ok
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub async fn test_submission(
    submission: SubmissionTask,
    pool: Data<PgPool>,
) -> Result<(), BoxDynError> {
    let submission_id = submission.id;

    log::info!("Test submission #{submission_id}");

    log::info!("Update total verdict");
    database::submissions::update_total_testing_result(
        &pool,
        submission_id,
        &TotalVerdict::Compiling,
        0,
    )
    .await
    .map_db(&pool, submission_id)
    .await?;

    let problem_id = submission.problem_id;
    let problem_path = submission.problem_path.clone();
    let run_path = submission.run_dir.clone();

    log::info!("Load problem's config");
    let config: ProblemConfig = get_problem_by_id(&pool, problem_id)
        .await
        .map_db(&pool, submission_id)
        .await?
        .into();
    log::info!("Loaded config: {config:?}");

    log::info!("Check subgroups' for correctness");
    assert_subgroups_correctness(&config)
        .map_db(&pool, submission_id)
        .await?;

    log::info!("Create tests paths");
    let tests_paths = TestsPaths::new(&run_path, &config, &submission.lang);

    log::info!("Compile solution");
    compile_solution(&tests_paths, &submission)
        .await
        .map_db(&pool, submission_id)
        .await?;

    log::info!("Prepare test env");
    prepare_test_env(problem_path, &config, &tests_paths)
        .await
        .map_db(&pool, submission_id)
        .await?;

    let mut total_score = 0;
    let mut subgroups_results: Vec<SubgroupResult> = Vec::with_capacity(config.subgroups.len());

    log::info!("Test solution on subgroups");
    database::submissions::update_total_testing_result(
        &pool,
        submission_id,
        &TotalVerdict::Testing,
        0,
    )
    .await?;
    for (i, subgroup) in config.subgroups.clone().iter().enumerate() {
        log::info!("Test on subgroup #{i}");
        log::info!("Insert a subgroup's testing result");

        let subgroup_index = i as i32;

        database::submissions::insert_subgroup_testing_result(&pool, submission_id, subgroup_index)
            .await
            .map_db(&pool, submission_id)
            .await?;

        let mut subgroup_result = SubgroupResult::default();

        log::info!("Check if subgroup needs to be tested on");
        if does_subgroup_need_to_be_tested_on(subgroup, &subgroups_results) {
            log::info!("Test solution on tests");
            write_subgroup_result(&mut subgroup_result, subgroup, &config, &tests_paths)
                .await
                .map_db(&pool, submission_id)
                .await?;
        } else {
            log::info!("Subgroup doesn't need to be tested, skip testing");
            subgroup_result.subgroup_verdict = SubgroupVerdict::Skipped;
        }

        total_score += subgroup_result.score;
        subgroups_results.push(subgroup_result.clone());

        log::info!("Update subgroup's testing result record");
        database::submissions::update_subgroup_testing_result(
            &pool,
            submission_id,
            subgroup_index,
            &subgroup_result,
        )
        .await
        .map_db(&pool, submission_id)
        .await?;
    }

    log::info!("Update total testing result");
    database::submissions::update_total_testing_result(
        &pool,
        submission_id,
        &match total_score {
            100 => TotalVerdict::Ok,
            _ => TotalVerdict::PartialSolution,
        },
        total_score,
    )
    .await?;

    Ok(())
}
