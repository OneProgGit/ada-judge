use ::tools::map::MapLogExt;
use apalis::prelude::{BoxDynError, Data};
use checker_runner::run_checker;
use database::{
    insert_subgroup_testing_result, update_subgroup_testing_result, update_total_testing_result,
};
use models::verdicts::TotalVerdict;
use models::{testing::*, verdicts::SubgroupVerdict};
use problem_config::ProblemConfig;
use solution_compiler::compile_solution;
use solution_runner::run_solution;
use sqlx::PgPool;
use std::path::Path;
use test_env_preparer::prepare_test_env;
use tokio::fs::read_to_string;
use tools::MapDbExt;

mod checker_runner;
mod constants;
mod problem_config;
mod solution_compiler;
mod solution_runner;
mod test_env_preparer;
pub mod tools;

async fn run_single_test(
    config: &ProblemConfig,
    tests_paths: &TestsPaths,
    test_id: i32,
) -> Result<CheckerResult, TotalVerdict> {
    let test_path = tests_paths.tests.join(test_id.to_string());

    let input_path = test_path.join("in");
    let answer_path = test_path.join("out");

    log::info!("Run solution");
    let solution_verdict = run_solution(config, &input_path, tests_paths).await?;

    if solution_verdict != SubgroupVerdict::Ok {
        log::error!("Run result isn't OK");
        return Ok(CheckerResult {
            verdict: solution_verdict,
            checker_msg: String::default(),
        });
    }

    log::info!("Run checker");
    run_checker(config, &input_path, answer_path, tests_paths).await
}

async fn load_config(problem_path: &Path) -> Result<ProblemConfig, TotalVerdict> {
    let config_text = read_to_string(problem_path.join("config.toml"))
        .await
        .map_log(TotalVerdict::InvalidProblem)?;

    toml::from_str::<ProblemConfig>(&config_text).map_log(TotalVerdict::InvalidProblem)
}

pub async fn test_submission(
    submission: SubmissionTask,
    pool: Data<PgPool>,
) -> Result<(), BoxDynError> {
    let submission_id = submission.id;

    log::info!("Test submission #{submission_id}");

    log::info!("Update total verdict");
    update_total_testing_result(&pool, submission_id, &TotalVerdict::Compiling, 0)
        .await
        .map_db(&pool, submission_id)
        .await?;

    let problem_path = submission.problem_path.clone();
    let run_path = submission.run_dir.clone();

    log::info!("Load problem's config");
    let config = load_config(&problem_path)
        .await
        .map_db(&pool, submission_id)
        .await?;

    log::info!("Check subgroups' for correctness");
    for (i, group) in config.test_groups.iter().enumerate() {
        log::info!("Check subgroup #{i} for correctness");
        if let Some(depends_on) = group.depends_on.clone() {
            for x in depends_on {
                if x >= i {
                    log::error!("Subgroup depends on a subgroup that has index less than it's");
                    update_total_testing_result(
                        &pool,
                        submission_id,
                        &TotalVerdict::InvalidProblem,
                        0,
                    )
                    .await?;
                    return Err(TotalVerdict::InvalidProblem.into());
                }
            }
        }
    }

    log::info!("Create tests paths");
    let tests_paths = TestsPaths::new(&run_path, &submission.lang);

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
    let mut groups_result: Vec<GroupResult> = Vec::with_capacity(config.test_groups.len());

    log::info!("Test solution on subgroups");
    update_total_testing_result(&pool, submission_id, &TotalVerdict::Testing, 0).await?;
    for (i, test_group) in config.test_groups.clone().iter().enumerate() {
        log::info!("Test on subgroup #{i}");
        log::info!("Insert a subgroup's testing result");

        let subgroup_testing_result_id =
            insert_subgroup_testing_result(&pool, i as i64, submission_id)
                .await
                .map_db(&pool, submission_id)
                .await?;

        let mut test_result = GroupResult {
            verdict: SubgroupVerdict::Ok,
            test: 0,
            score: test_group.score,
            checker_msg: String::new(),
        };

        log::info!("Check subgroup's dependencies");
        if let Some(depends_on) = &test_group.depends_on {
            for i in depends_on {
                if groups_result[*i].verdict != SubgroupVerdict::Ok {
                    log::error!("Subgroup's dependency isn't OK, skip testing");
                    test_result.verdict = SubgroupVerdict::Skipped;
                    test_result.score = 0;
                    break;
                }
            }
        }

        if test_result.verdict != SubgroupVerdict::Skipped {
            log::info!("Test solution on tests");
            for test_id in &test_group.tests {
                let test_id = *test_id;
                log::info!("Run test #{test_id}");

                test_result.test = test_id;
                let run_result = run_single_test(&config, &tests_paths, test_id).await;

                match run_result {
                    Err(verdict) => {
                        log::error!("{verdict}");
                        update_total_testing_result(&pool, submission_id, &verdict, 0)
                            .await
                            .map_db(&pool, submission_id)
                            .await?;
                    }
                    Ok(value) => {
                        test_result.verdict = value.verdict;
                        test_result.test = test_id;
                        test_result.checker_msg = value.checker_msg;
                    }
                }

                if test_result.verdict != SubgroupVerdict::Ok {
                    log::error!("Verdict isn't OK, skip testing");
                    test_result.score = 0;
                    break;
                }
            }
        }

        total_score += test_result.score;
        groups_result.push(test_result.clone());

        log::info!("Update subgroup's testing result record");
        update_subgroup_testing_result(
            &pool,
            subgroup_testing_result_id,
            &test_result.verdict,
            test_result.test,
            test_result.score,
            test_result.checker_msg,
        )
        .await
        .map_db(&pool, submission_id)
        .await?;
    }

    log::info!("Update total testing result");
    update_total_testing_result(
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
