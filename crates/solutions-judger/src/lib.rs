use crate::checker_runner::run_checker;
use crate::db::{
    insert_subgroup_testing_result, insert_submission, update_subgroup_testing_result,
    update_total_testing_result,
};
use crate::problem_config::ProblemConfig;
use crate::solution_compiler::compile_solution;
use crate::solution_runner::run_solution;
use crate::test_env_preparer::prepare_test_env;
use crate::tools::{MapDbExt, MapLogExt};
use apalis::prelude::{BoxDynError, Data, TaskSink};
use axum::body::Bytes;
use axum::extract::Multipart;
use axum::{Json, extract::State};
use models::AppState;
use models::verdicts::TotalVerdict;
use models::{testing::*, verdicts::SubgroupVerdict};
use sqlx::PgPool;
use std::{path::PathBuf, sync::Arc};
use tokio::fs::{self, File, read_to_string};
use tokio::io::AsyncWriteExt;

mod checker_runner;
mod constants;
pub mod db;
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

pub async fn test(submission: SubmissionTask, pool: Data<PgPool>) -> Result<(), BoxDynError> {
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
    let config_text = read_to_string(problem_path.join("config.toml"))
        .await
        .map_log(TotalVerdict::InvalidProblem)
        .map_db(&pool, submission_id)
        .await?;

    let config = toml::from_str::<ProblemConfig>(&config_text)
        .map_log(TotalVerdict::InvalidProblem)
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

        log::info!("Update subgroup's test result record");
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

    log::info!("Update total test result");
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

pub async fn push_submission_to_queue(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> Result<Json<i64>, Json<TotalVerdict>> {
    let mut submission: Option<Submission> = None;
    let mut file_stream: Option<Bytes> = None;

    log::info!("Extracting submission data and file");
    while let Some(field) = multipart
        .next_field()
        .await
        .map_log(TotalVerdict::InvalidRequest)?
    {
        match field.name() {
            Some("submission_data") => {
                let text = field.text().await.map_log(TotalVerdict::InvalidRequest)?;
                submission =
                    Some(serde_json::from_str(&text).map_log(TotalVerdict::InvalidRequest)?);
            }
            Some("submission_file") => {
                file_stream = Some(field.bytes().await.map_log(TotalVerdict::Bug)?);
            }
            _ => {}
        }
    }

    let submission = match submission {
        Some(submission) => submission,
        None => {
            log::error!("No submission data was provided");
            return Err(Json(TotalVerdict::InvalidRequest));
        }
    };

    let file_stream = match file_stream {
        Some(file_stream) => file_stream,
        None => {
            log::error!("No submission files were provided");
            return Err(Json(TotalVerdict::InvalidRequest));
        }
    };

    // TODO: replace id with real user id and problem path with problem id

    log::info!("Push to queue: {submission:?}");

    let submission_id = insert_submission(&state.db, &submission.problem_path).await?;

    let run_dir = PathBuf::from("/submissions_envs").join(submission_id.to_string());
    fs::create_dir(run_dir.clone())
        .await
        .map_log(TotalVerdict::Bug)?;
    // TODO: Add support for other languages
    let run_path = run_dir.join(format!("run.{}", get_lang_str(&submission.lang)));
    let mut run_file = File::create(run_path).await.map_log(TotalVerdict::Bug)?;
    run_file
        .write_all(&file_stream)
        .await
        .map_log(TotalVerdict::Bug)?;

    let submission_task = SubmissionTask {
        problem_path: submission.problem_path,
        run_dir,
        lang: submission.lang,
        id: submission_id,
    };

    state
        .apalis_backend
        .lock()
        .await
        .push(submission_task)
        .await
        .map_log(TotalVerdict::Bug)?;

    Ok(Json(submission_id))
}
