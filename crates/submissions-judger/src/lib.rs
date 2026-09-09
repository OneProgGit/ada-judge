#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![deny(warnings)]
#![forbid(unsafe_code)]
#![allow(clippy::too_many_lines, clippy::missing_errors_doc)]

use crate::{
    checker_runner::{get_checker_run_twice_interactive_verdict, get_checker_run_twice_verdict},
    constants::EPS,
    interactive_runner::{get_interactive_run_twice_verdict, get_interactive_verdict},
    solution_runner::get_solution_verdict,
};
use aj_models::{
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
use sqlx::PgPool;
use tokio::fs::File;

mod checker_runner;
mod constants;
mod interactive_runner;
mod solution_compiler;
mod solution_runner;

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
                    .map_err(|_| TestingVerdict::Fail)?;
                _ = File::create(tests_paths.output.clone())
                    .await
                    .map_err(|_| TestingVerdict::Fail)?;
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
                    .map_err(|_| TestingVerdict::Fail)?;
                _ = File::create(tests_paths.output.clone())
                    .await
                    .map_err(|_| TestingVerdict::Fail)?;
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
                    .map_err(|_| TestingVerdict::Fail)?;
            }

            let checker_verdict =
                get_checker_run_twice_verdict(config, &answer_path, tests_paths, 1).await?;

            if checker_verdict != Verdict::Ok {
                return Ok(checker_verdict);
            }

            {
                _ = File::create(tests_paths.output.clone())
                    .await
                    .map_err(|_| TestingVerdict::Fail)?;
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
                    .map_err(|_| TestingVerdict::Fail)?;
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
                    .map_err(|_| TestingVerdict::Fail)?;
                _ = File::create(tests_paths.output.clone())
                    .await
                    .map_err(|_| TestingVerdict::Fail)?;
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
                    .map_err(|_| TestingVerdict::Fail)?;
                _ = File::create(tests_paths.output.clone())
                    .await
                    .map_err(|_| TestingVerdict::Fail)?;
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
                    .map_err(|_| TestingVerdict::Fail)?;
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
    subgroup_result: &mut SubgroupResult,
) -> Result<(), (TestingVerdict, i32)> {
    let mut score = 0.;
    let per_test_scoring = subgroup.score_per_test.is_some();
    let mut ok = true;
    for test in &subgroup.tests {
        let test = *test;

        database::submissions::create_test_result(
            pool,
            submission_id,
            test,
            if per_test_scoring { Some(0.) } else { None },
        )
        .await
        .map_err(|_| (TestingVerdict::Fail, test))?;

        if !ok {
            let test_result = TestResult {
                verdict: Verdict::Skipped,
                score: if per_test_scoring { Some(0.) } else { None },
            };

            database::submissions::update_test_testing_result(
                pool,
                submission_id,
                test,
                &test_result,
            )
            .await
            .map_err(|_| (TestingVerdict::Fail, test))?;

            continue;
        }

        let test_verdict = get_test_verdict(config, tests_paths, test)
            .await
            .map_err(|e| (e, test))?;

        subgroup_result.test = test;
        subgroup_result.verdict = test_verdict.clone();

        let test_result = TestResult {
            verdict: test_verdict,
            score: if subgroup_result.verdict == Verdict::Ok {
                subgroup.score_per_test
            } else if per_test_scoring {
                Some(0.)
            } else {
                None
            },
        };

        database::submissions::update_test_testing_result(pool, submission_id, test, &test_result)
            .await
            .map_err(|_| (TestingVerdict::Fail, test))?;

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
            subgroup_result.score = subgroup.score.ok_or((TestingVerdict::Fail, 0))?;
        }
    }

    Ok(())
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss
)]
pub async fn test_submission(
    submission: SubmissionTask,
    pool: Data<PgPool>,
) -> Result<(), BoxDynError> {
    let submission_id = submission.id;
    database::submissions::update_submission(&pool, submission_id, &TestingVerdict::Compiling, 0.)
        .await
        .map_db(&pool, submission_id, None)
        .await?;

    let problem_id = submission.problem_id;
    let run_path = submission.run_dir.clone();
    let config = get_problem(&pool, problem_id)
        .await
        .map_db(&pool, submission_id, None)
        .await?
        .into();
    let tests_paths = TestsPaths::new(
        &submission.problem_path,
        &run_path,
        &config,
        &submission.language,
    );
    compile_solution(&run_path, &tests_paths, &submission)
        .await
        .map_db(&pool, submission_id, None)
        .await?;

    let mut total_score = 0.;
    let mut max_score = 0.;
    let mut subgroups_results: Vec<SubgroupResult> = Vec::with_capacity(config.subgroups.len());
    database::submissions::update_submission(&pool, submission_id, &TestingVerdict::Testing, 0.)
        .await?;
    for (i, subgroup) in config.subgroups.clone().iter().enumerate() {
        let subgroup_index = i as i32;
        max_score += subgroup.score.unwrap_or_else(|| {
            subgroup.score_per_test.map_or_else(
                || unreachable!(),
                |score_per_test| score_per_test * (subgroup.tests.len() as f64),
            )
        });
        let mut subgroup_result = SubgroupResult {
            verdict: Verdict::Testing,
            test: 0,
            score: 0.,
        };
        database::submissions::create_subgroup_result(&pool, submission_id, subgroup_index)
            .await
            .map_db(&pool, submission_id, None)
            .await
            .map_err(|_| TestingVerdict::Fail)?;

        if subgroup.should_skip(&subgroups_results) {
            subgroup_result.verdict = Verdict::Skipped;

            let per_test = subgroup.score_per_test.is_some();
            for test_id in &subgroup.tests {
                let test_id = *test_id;
                database::submissions::create_test_result(
                    &pool,
                    submission_id,
                    test_id,
                    if per_test { Some(0.) } else { None },
                )
                .await?;

                let test_result = TestResult {
                    verdict: Verdict::Skipped,
                    score: if per_test { Some(0.) } else { None },
                };

                database::submissions::update_test_testing_result(
                    &pool,
                    submission_id,
                    test_id,
                    &test_result,
                )
                .await?;
            }
        } else if let Err((e, test)) = test_subgroup(
            &pool,
            submission_id,
            subgroup,
            &config,
            &tests_paths,
            &mut subgroup_result,
        )
        .await
        {
            Err(e)
                .map_db(&pool, submission_id, Some((subgroup_index, test)))
                .await
                .map_err(|_| TestingVerdict::Fail)?;
        }

        total_score += subgroup_result.score;
        subgroups_results.push(subgroup_result.clone());

        database::submissions::update_subgroup_result(
            &pool,
            submission_id,
            subgroup_index,
            &subgroup_result,
        )
        .await
        .map_db(&pool, submission_id, None)
        .await?;
    }
    database::submissions::update_submission(
        &pool,
        submission_id,
        if (total_score - max_score).abs() < EPS {
            &TestingVerdict::Ok
        } else {
            &TestingVerdict::PartialSolution
        },
        total_score,
    )
    .await?;

    Ok(())
}
