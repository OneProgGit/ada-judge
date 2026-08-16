#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![deny(warnings)]
#![deny(missing_docs)]
#![deny(rustdoc::all)]
#![deny(rustdoc::broken_intra_doc_links)]
#![forbid(unsafe_code)]

use crate::{
    checker_runner::{get_checker_run_twice_interactive_verdict, get_checker_run_twice_verdict},
    interactive_runner::{get_interactive_run_twice_verdict, get_interactive_verdict},
};
use ::tools::map::MapLogExt;
use aj_models::{
    errors::AdaJudgeError,
    problems::{ProblemConfig, ProblemType, Subgroup, SubgroupType},
    testing::{SubgroupResult, TestResult},
    verdicts::{TestingVerdict, Verdict},
};
use apalis::prelude::{BoxDynError, Data};
use checker_runner::get_checker_verdict;
use database::problems::get_problem;
use database::tools::MapDbExt;
use models::testing::{SubmissionTask, TestsPaths};
use solution_compiler::compile_solution;
use solution_runner::get_solution_verdict;
use sqlx::PgPool;
use tokio::fs::File;

mod checker_runner;
mod constants;
mod interactive_runner;
mod solution_compiler;
mod solution_runner;
mod tools;

async fn get_test_verdict(
    config: &ProblemConfig,
    tests_paths: &TestsPaths,
    test_id: i32,
) -> Result<Verdict, TestingVerdict> {
    let test_path = tests_paths.tests.join(test_id.to_string());

    let input_path = test_path.join("in");
    let answer_path = test_path.join("out");

    match config.r#type {
        ProblemType::Default => {
            {
                _ = File::create(tests_paths.input.clone())
                    .await
                    .map_log(TestingVerdict::InvalidProblem)?;
                _ = File::create(tests_paths.output.clone())
                    .await
                    .map_log(TestingVerdict::InvalidProblem)?;
            }

            let solution_verdict = get_solution_verdict(config, &input_path, tests_paths).await?;

            if solution_verdict != Verdict::Ok {
                return Ok(solution_verdict);
            }

            get_checker_verdict(config, &input_path, &answer_path, tests_paths).await
        }
        ProblemType::Interactive => {
            get_interactive_verdict(config, &answer_path, tests_paths).await
        }
        ProblemType::RunTwice => {
            {
                _ = File::create(tests_paths.input.clone())
                    .await
                    .map_log(TestingVerdict::InvalidProblem)?;
                _ = File::create(tests_paths.output.clone())
                    .await
                    .map_log(TestingVerdict::InvalidProblem)?;
            }

            let checker_verdict =
                get_checker_run_twice_verdict(config, &answer_path, tests_paths, 0).await?;

            if checker_verdict != Verdict::Ok {
                return Ok(checker_verdict);
            }

            let solution_verdict =
                get_solution_verdict(config, &tests_paths.input, tests_paths).await?;

            if solution_verdict != Verdict::Ok {
                return Ok(solution_verdict);
            }

            {
                _ = File::create(tests_paths.input.clone())
                    .await
                    .map_log(TestingVerdict::InvalidProblem)?;
            }

            let checker_verdict =
                get_checker_run_twice_verdict(config, &answer_path, tests_paths, 1).await?;

            if checker_verdict != Verdict::Ok {
                return Ok(checker_verdict);
            }

            {
                _ = File::create(tests_paths.output.clone())
                    .await
                    .map_log(TestingVerdict::InvalidProblem)?;
            }

            let solution_verdict =
                get_solution_verdict(config, &tests_paths.input, tests_paths).await?;

            if solution_verdict != Verdict::Ok {
                return Ok(solution_verdict);
            }

            get_checker_run_twice_verdict(config, &answer_path, tests_paths, 2).await
        }
        ProblemType::InteractiveRunTwice => {
            {
                _ = File::create(tests_paths.input.clone())
                    .await
                    .map_log(TestingVerdict::InvalidProblem)?;
            }

            let checker_verdict = get_interactive_run_twice_verdict(
                config,
                &answer_path,
                &tests_paths.input,
                tests_paths,
                0,
            )
            .await?;

            if checker_verdict != Verdict::Ok {
                return Ok(checker_verdict);
            }

            get_interactive_run_twice_verdict(
                config,
                &answer_path,
                &tests_paths.input,
                tests_paths,
                1,
            )
            .await
        }
        ProblemType::RunTwiceFirstInteractive => {
            {
                _ = File::create(tests_paths.input.clone())
                    .await
                    .map_log(TestingVerdict::InvalidProblem)?;
                _ = File::create(tests_paths.output.clone())
                    .await
                    .map_log(TestingVerdict::InvalidProblem)?;
            }

            let checker_verdict = get_interactive_run_twice_verdict(
                config,
                &answer_path,
                &tests_paths.input,
                tests_paths,
                0,
            )
            .await?;

            if checker_verdict != Verdict::Ok {
                return Ok(checker_verdict);
            }

            let solution_verdict =
                get_solution_verdict(config, &tests_paths.input, tests_paths).await?;

            if solution_verdict != Verdict::Ok {
                return Ok(solution_verdict);
            }

            get_checker_run_twice_interactive_verdict(
                config,
                &answer_path,
                &tests_paths.input,
                tests_paths,
                1,
            )
            .await
        }
        ProblemType::RunTwiceSecondInteractive => {
            {
                _ = File::create(tests_paths.input.clone())
                    .await
                    .map_log(TestingVerdict::InvalidProblem)?;
                _ = File::create(tests_paths.output.clone())
                    .await
                    .map_log(TestingVerdict::InvalidProblem)?;
            }
            let checker_verdict = get_checker_run_twice_interactive_verdict(
                config,
                &answer_path,
                &tests_paths.output,
                tests_paths,
                0,
            )
            .await?;

            if checker_verdict != Verdict::Ok {
                return Ok(checker_verdict);
            }

            {
                _ = File::create(tests_paths.output.clone())
                    .await
                    .map_log(TestingVerdict::InvalidProblem)?;
            }

            let solution_verdict =
                get_solution_verdict(config, &tests_paths.input, tests_paths).await?;

            if solution_verdict != Verdict::Ok {
                return Ok(solution_verdict);
            }

            get_interactive_run_twice_verdict(
                config,
                &answer_path,
                &tests_paths.output,
                tests_paths,
                1,
            )
            .await
        }
    }
}

async fn test_subgroup(
    pool: &PgPool,
    submission_id: i64,
    subgroup: &Subgroup,
    config: &ProblemConfig,
    tests_paths: &TestsPaths,
) -> Result<(), TestingVerdict> {
    database::submissions::insert_subgroup_testing_result(&pool, submission_id, subgroup_index)
        .await
        .map_db(&pool, submission_id)
        .await?;

    let mut subgroup_result = SubgroupResult {
        verdict: Verdict::Testing,
        test: 0,
        score: 0,
    };

    let mut score = 0;
    let per_test_scoring = subgroup.score_per_test.is_some();
    let mut ok = true;
    for test_id in &subgroup.tests {
        let test_id = *test_id;

        database::submissions::insert_test_testing_result(
            &pool,
            submission_id,
            test_id,
            if per_test_scoring { Some(0) } else { None },
        )
        .await?;

        if !ok {
            let test_result = TestResult {
                verdict: Verdict::Skipped,
                score: if per_test_scoring { Some(0) } else { None },
            };

            database::submissions::update_test_testing_result(
                &pool,
                submission_id,
                test_id,
                &test_result,
            )
            .await?;

            continue;
        }

        let test_verdict = get_test_verdict(config, tests_paths, test_id).await?;

        subgroup_result.test = test_id;
        subgroup_result.verdict = test_verdict.clone();

        let test_result = TestResult {
            verdict: test_verdict,
            score: if subgroup_result.verdict == Verdict::Ok {
                subgroup.score_per_test
            } else if per_test_scoring {
                Some(0)
            } else {
                None
            },
        };

        database::submissions::update_test_testing_result(
            &pool,
            submission_id,
            test_id,
            &test_result,
        )
        .await?;

        if subgroup_result.verdict != Verdict::Ok && !per_test_scoring {
            ok = false;
        } else if per_test_scoring && subgroup_result.verdict == Verdict::Ok {
            score += subgroup.score_per_test.unwrap();
        }
    }
    if ok && subgroup.r#type != SubgroupType::Sample {
        if per_test_scoring {
            subgroup_result.score = score;
        } else {
            subgroup_result.score = subgroup.score.ok_or(TestingVerdict::Bug)?;
        }
    }

    Ok(())
}

#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
pub async fn test_submission(
    submission: SubmissionTask,
    pool: Data<PgPool>,
) -> Result<(), BoxDynError> {
    let submission_id = submission.id;
    database::submissions::update_total_testing_result(
        &pool,
        submission_id,
        &TestingVerdict::Compiling,
        0,
    )
    .await
    .map_db(&pool, submission_id)
    .await?;

    let problem_id = submission.problem_id;
    let run_path = submission.run_dir.clone();
    let config = get_problem(&pool, problem_id)
        .await
        .map_db(&pool, submission_id)
        .await?;
    let tests_paths = TestsPaths::new(
        &submission.problem_path,
        &run_path,
        &config,
        &submission.language,
    );
    compile_solution(&run_path, &tests_paths, &submission)
        .await
        .map_db(&pool, submission_id)
        .await?;

    let mut total_score = 0;
    let mut subgroups_results: Vec<SubgroupResult> = Vec::with_capacity(config.subgroups.len());
    database::submissions::update_total_testing_result(
        &pool,
        submission_id,
        &TestingVerdict::Testing,
        0,
    )
    .await?;
    for (i, subgroup) in config.subgroups.clone().iter().enumerate() {
        let subgroup_index = i as i32;

        if !subgroup.should_skip(&subgroups_results) {
            test_subgroup(&pool, submission_id, subgroup, &config, &tests_paths)
                .await
                .map_db(&pool, submission_id)
                .await?;
        } else {
            subgroup_result.verdict = Verdict::Skipped;

            let per_test = subgroup.score_per_test.is_some();
            for test_id in &subgroup.tests {
                let test_id = *test_id;
                database::submissions::insert_test_testing_result(
                    &pool,
                    submission_id,
                    test_id,
                    if per_test { Some(0) } else { None },
                )
                .await?;

                let test_result = TestResult {
                    verdict: Verdict::Skipped,
                    score: if per_test { Some(0) } else { None },
                };

                database::submissions::update_test_testing_result(
                    &pool,
                    submission_id,
                    test_id,
                    &test_result,
                )
                .await?;
            }
        }

        total_score += subgroup_result.score;
        subgroups_results.push(subgroup_result.clone());

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
    database::submissions::update_total_testing_result(
        &pool,
        submission_id,
        &match total_score {
            100 => TestingVerdict::Ok,
            _ => TestingVerdict::PartialSolution,
        },
        total_score,
    )
    .await?;

    Ok(())
}
