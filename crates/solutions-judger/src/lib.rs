use crate::{constants::*, problem_config::ProblemConfig};
use apalis::prelude::{BoxDynError, Data, TaskSink};
use axum::{Json, extract::State};
use fs_extra::dir::CopyOptions;
use models::AppState;
use models::verdicts::TotalVerdict;
use models::{error::Error, testing::*, verdicts::SubgroupVerdict};
use sqlx::PgPool;
use std::env;
use std::{
    fs::{self, File, read_to_string},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::Arc,
    time::Duration,
};
use wait_timeout::ChildExt;

pub mod constants;
pub mod problem_config;

fn prepare_test_env(
    problem_path: PathBuf,
    config: &ProblemConfig,
    tests_paths: &TestsPaths,
) -> Result<(), Error> {
    log::info!("Copy checker");
    fs::copy(
        problem_path.join(config.checker.path.clone()),
        tests_paths.checker.clone(),
    )
    .map_err(|e| {
        log::error!("{e}");
        Error::InvalidProblem
    })?;

    let mut opt = CopyOptions::new();
    opt.overwrite = true;
    opt.copy_inside = true;
    opt.content_only = false;

    log::info!("Copy tests");
    fs_extra::dir::copy(
        problem_path.join(config.tests.path.clone()),
        tests_paths.tests.clone(),
        &opt,
    )
    .map_err(|e| {
        log::error!("{e}");
        Error::InvalidProblem
    })?;

    log::info!("Create stderr file");
    fs::write(tests_paths.error.clone(), "").map_err(|_| Error::InvalidProblem)?;

    Ok(())
}

fn convert_path_in_container_to_path_in_host(path: &Path) -> Result<PathBuf, Error> {
    if let Ok(host_run_dir) = env::var("HOST_RUNS_DIR") {
        let host_runs_dir = PathBuf::from(host_run_dir);
        Ok(host_runs_dir.join(path.strip_prefix("/").map_err(|e| {
            log::error!("{e}");
            Error::InvalidProblem
        })?))
    } else {
        Ok(path.into())
    }
}

fn run_solution(
    config: &ProblemConfig,
    input_path: &Path,
    tests_paths: &TestsPaths,
) -> Result<SubgroupVerdict, Error> {
    log::info!("Open stdin file");
    let stdin_file = File::open(input_path).map_err(|e| {
        log::error!("{e}");
        Error::InvalidProblem
    })?;
    log::info!("Open stdout file");
    let stdout_file = File::create(tests_paths.output.clone()).map_err(|e| {
        log::error!("{e}");
        Error::InvalidProblem
    })?;
    log::info!("Open stderr file");
    let stderr_file = File::create(tests_paths.error.clone()).map_err(|e| {
        log::error!("{e}");
        Error::InvalidProblem
    })?;

    log::info!("Run solution cmd");
    let mut solution_cmd = Command::new("docker")
        .args([
            "run",
            "--rm",
            "--network",
            "none",
            "--memory",
            &format!("{}m", config.limits.memory_limit_mb),
            "--cpus",
            "0.3",
            "--pids-limit",
            "32",
            "--read-only",
            "--cap-drop",
            "ALL",
            "-i",
            "--security-opt",
            "no-new-privileges",
            "-v",
            &format!(
                "{}:/sandbox/bin:ro",
                convert_path_in_container_to_path_in_host(&tests_paths.solution)?.display()
            ),
            "-w",
            "/sandbox",
            "sandbox-runner",
            "/sandbox/bin",
        ])
        .stdin(Stdio::from(stdin_file))
        .stdout(Stdio::from(stdout_file))
        .stderr(Stdio::from(stderr_file))
        .spawn()
        .map_err(|e| {
            log::error!("{e}");
            Error::Bug
        })?;

    let timeout = Duration::from_millis(config.limits.time_limit_ms);
    let solution_status = solution_cmd.wait_timeout(timeout).map_err(|e| {
        log::error!("{e}");
        Error::Bug
    })?;

    _ = solution_cmd.kill();
    log::info!("Check solution status");
    match solution_status {
        None => Ok(SubgroupVerdict::TimeLimitExceeded),
        Some(status) => match status.code() {
            Some(VERDICT_OK) => Ok(SubgroupVerdict::Ok),
            Some(VERDICT_MLE) => Ok(SubgroupVerdict::MemoryLimitExceeded),
            _ => Ok(SubgroupVerdict::RuntimeError),
        },
    }
}

fn run_checker(
    config: &ProblemConfig,
    input_path: &Path,
    answer_path: PathBuf,
    tests_paths: &TestsPaths,
) -> Result<CheckerResult, Error> {
    log::info!("Open stderr file");

    let stderr_file = File::create(tests_paths.error.clone()).map_err(|e| {
        log::error!("{e}");
        Error::InvalidProblem
    })?;

    log::info!("Run checker cmd");
    let mut checker_cmd = Command::new("docker")
        .args([
            "run",
            "--rm",
            "--network",
            "none",
            "--memory",
            &format!("{}m", config.limits.memory_limit_mb),
            "--cpus",
            "0.3",
            "--pids-limit",
            "32",
            "--read-only",
            "--cap-drop",
            "ALL",
            "-i",
            "--security-opt",
            "no-new-privileges",
            "-v",
            &format!(
                "{}:/sandbox/bin:ro",
                convert_path_in_container_to_path_in_host(&tests_paths.checker)?.display()
            ),
            "-v",
            &format!(
                "{}:/sandbox/input:ro",
                convert_path_in_container_to_path_in_host(input_path)?.display()
            ),
            "-v",
            &format!(
                "{}:/sandbox/output:ro",
                convert_path_in_container_to_path_in_host(&tests_paths.output)?.display()
            ),
            "-v",
            &format!(
                "{}:/sandbox/answer:ro",
                convert_path_in_container_to_path_in_host(&answer_path)?.display()
            ),
            "-w",
            "/sandbox",
            "sandbox-runner",
            "/sandbox/bin",
            "/sandbox/input",
            "/sandbox/output",
            "/sandbox/answer",
        ])
        .stderr(Stdio::from(stderr_file))
        .spawn()
        .map_err(|e| {
            log::error!("{e}");
            Error::Bug
        })?;

    let timeout = Duration::from_millis(config.limits.time_limit_ms);
    let checker_status = checker_cmd.wait_timeout(timeout).map_err(|e| {
        log::error!("{e}");
        Error::Bug
    })?;

    _ = checker_cmd.kill();
    log::info!("Check checker status");
    match checker_status {
        None => Err(Error::CheckerFailed),
        Some(status) => {
            let checker_msg = fs::read_to_string(tests_paths.error.clone()).map_err(|e| {
                log::error!("{e}");
                Error::InvalidProblem
            })?;

            match status.code() {
                Some(CHECKER_OK) => Ok(CheckerResult {
                    verdict: SubgroupVerdict::Ok,
                    checker_msg,
                }),
                Some(CHECKER_WA) => Ok(CheckerResult {
                    verdict: SubgroupVerdict::WrongAnswer,
                    checker_msg,
                }),
                Some(CHECKER_PE) => Ok(CheckerResult {
                    verdict: SubgroupVerdict::PresentationError,
                    checker_msg,
                }),
                _ => Err(Error::CheckerFailed),
            }
        }
    }
}

fn run_single_test(
    config: &ProblemConfig,
    tests_paths: &TestsPaths,
    test_id: i32,
) -> Result<CheckerResult, Error> {
    let test_path = tests_paths.tests.join(test_id.to_string());

    let input_path = test_path.join("in");
    let answer_path = test_path.join("out");

    log::info!("Run solution");
    let solution_verdict = run_solution(config, &input_path, tests_paths)?;

    if solution_verdict != SubgroupVerdict::Ok {
        log::error!("Run result isn't OK");
        return Ok(CheckerResult {
            verdict: solution_verdict,
            checker_msg: String::default(),
        });
    }

    log::info!("Run checker");
    run_checker(config, &input_path, answer_path, tests_paths)
}

async fn update_total_testing_verdict(
    pool: &PgPool,
    id: i64,
    verdict: TotalVerdict,
) -> Result<(), Error> {
    sqlx::query("update submissions set total_verdict = $1 where id = $2")
        .bind(verdict)
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| {
            log::error!("{e}");
            Error::Bug
        })?;
    Ok(())
}

pub async fn test(submission: SubmissionTask, pool: Data<PgPool>) -> Result<(), BoxDynError> {
    let id = submission.id;

    log::info!("Test submission #{id}");

    log::info!("Update total verdict");
    update_total_testing_verdict(&pool, id, TotalVerdict::Testing).await?;

    log::info!("Canonicalize problem path");
    let problem_path = submission.problem_path.canonicalize();
    let problem_path = match problem_path {
        Err(e) => {
            log::error!("{e}");
            update_total_testing_verdict(&pool, id, TotalVerdict::InvalidProblem).await?;
            return Err(Error::InvalidProblem.into());
        }
        Ok(val) => val,
    };

    log::info!("Canonicalize run path");
    let run_path = submission.run_path.canonicalize();
    let run_path = match run_path {
        Err(e) => {
            log::error!("{e}");
            update_total_testing_verdict(&pool, id, TotalVerdict::InvalidProblem).await?;
            return Err(Error::InvalidProblem.into());
        }
        Ok(val) => val,
    };

    log::info!("Load problem's config");
    let config_text = read_to_string(problem_path.join("config.toml"));
    let config_text = match config_text {
        Err(e) => {
            log::error!("{e}");
            update_total_testing_verdict(&pool, id, TotalVerdict::InvalidProblem).await?;
            return Err(Error::InvalidProblem.into());
        }
        Ok(val) => val,
    };

    let config = toml::from_str::<ProblemConfig>(&config_text);
    let config = match config {
        Err(e) => {
            log::error!("{e}");
            update_total_testing_verdict(&pool, id, TotalVerdict::InvalidProblem).await?;
            return Err(Error::InvalidProblem.into());
        }
        Ok(val) => val,
    };

    log::info!("Check subgroups' for correctness");
    for (i, group) in config.test_groups.iter().enumerate() {
        log::info!("Check subroup #{i} for correctness");
        if let Some(depends_on) = group.depends_on.clone() {
            for x in depends_on {
                if x >= i {
                    log::error!("Subgroup depends on a subgroup, which has index less than its");
                    update_total_testing_verdict(&pool, id, TotalVerdict::InvalidProblem).await?;
                    return Err(Error::InvalidProblem.into());
                }
            }
        }
    }

    log::info!("Create tests paths");
    let tests_paths = TestsPaths::new(&run_path);

    log::info!("Prepare test env");
    prepare_test_env(problem_path, &config, &tests_paths)?;

    let mut total_score = 0;
    let mut groups_result: Vec<GroupResult> = Vec::with_capacity(config.test_groups.len());

    log::info!("Test solution on subgroups");
    for (group_ind, test_group) in config.test_groups.clone().iter().enumerate() {
        log::info!("Test on subgroup #{group_ind}");
        log::info!("Insert a subgroup's testing result");

        let result_id: Result<i64, sqlx::Error> = sqlx::query_scalar(
            "insert into submissions_subgroups_results (subgroup_id, submission_id, verdict, test, score, checker_msg) values ($1, $2, $3, $4, $5, $6) returning id",
        )
        .bind(groups_result.len() as i64)
        .bind(id)
        .bind(SubgroupVerdict::Testing)
        .bind(0)
        .bind(0)
        .bind("")
        .fetch_one(&*pool)
        .await;
        let result_id = match result_id {
            Err(e) => {
                log::error!("{e}");
                update_total_testing_verdict(&pool, id, TotalVerdict::Bug).await?;
                return Err(Error::Bug.into());
            }
            Ok(val) => val,
        };

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
                let run_result = run_single_test(&config, &tests_paths, test_id);

                match run_result {
                    Err(_) => {
                        test_result.verdict = SubgroupVerdict::Bug;
                    }
                    Ok(val) => {
                        test_result.verdict = val.verdict;
                        test_result.test = test_id;
                        test_result.checker_msg = val.checker_msg;
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

        if let Err(e) = sqlx::query(
            "update submissions_subgroups_results set verdict = $1, test = $2, score = $3, checker_msg = $4 where id = $5",
        )
        .bind(test_result.verdict)
        .bind(test_result.test)
        .bind(test_result.score)
        .bind(test_result.checker_msg)
        .bind(result_id)
        .execute(&*pool)
        .await {
            log::error!("{e}");
            update_total_testing_verdict(&pool, id, TotalVerdict::Bug).await?;
            return Err(Error::Bug.into());
        }
    }

    log::info!("Update total test result");
    sqlx::query("update submissions set total_verdict = $1, total_score = $2 WHERE id = $3")
        .bind(match total_score {
            100 => TotalVerdict::Ok,
            _ => TotalVerdict::PartialSolution,
        })
        .bind(total_score)
        .bind(id)
        .execute(&*pool)
        .await
        .map_err(|e| {
            log::error!("{e}");
            Error::Bug
        })?;

    Ok(())
}

pub async fn push_submission_to_queue(
    State(state): State<Arc<AppState>>,
    Json(submission): Json<Submission>,
) -> Result<Json<i64>, Json<Error>> {
    // TODO: replace id with real user id and problem path with problem id

    log::info!("Push to queue: {submission:?}");

    let id: i64 = sqlx::query_scalar(
        "insert into submissions (problem_id, user_id, total_verdict, total_score) values ($1, $2, $3, $4) returning id",
    )
    .bind(
        submission
            .problem_path
            .to_str()
            .ok_or(Error::InvalidProblem)?,
    )
    .bind(100)
    .bind(TotalVerdict::Pending)
    .bind(0)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        log::error!("{e}");
        Error::Bug
    })?;

    let submission_task = SubmissionTask {
        problem_path: submission.problem_path,
        run_path: submission.run_path,
        id,
    };

    state
        .apalis_backend
        .lock()
        .await
        .push(submission_task)
        .await
        .map_err(|e| {
            log::error!("{e}");
            Error::Bug
        })?;

    Ok(Json(id))
}
