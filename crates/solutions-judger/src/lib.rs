use crate::{constants::*, problem_config::ProblemConfig};
use apalis::prelude::{BoxDynError, Data, TaskSink};
use axum::{Json, extract::State};
use fs_extra::dir::CopyOptions;
use models::AppState;
use models::enums::AdaJudgeTotalVerdict;
use models::{
    enums::{AdaJudgeError, AdaJudgeVerdict},
    testing::*,
};
use sqlx::PgPool;
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
) -> Result<(), AdaJudgeError> {
    eprintln!("Copy checker");
    fs::copy(
        problem_path.join(config.checker.path.clone()),
        tests_paths.checker.clone(),
    )
    .map_err(|e| {
        eprintln!("{e}");
        AdaJudgeError::InvalidProblem
    })?;

    let mut opt = CopyOptions::new();
    opt.overwrite = true;
    opt.copy_inside = true;
    opt.content_only = false;

    eprintln!("Copy tests");
    fs_extra::dir::copy(
        problem_path.join(config.tests.path.clone()),
        tests_paths.tests.clone(),
        &opt,
    )
    .map_err(|e| {
        eprintln!("{e}");
        AdaJudgeError::InvalidProblem
    })?;

    eprintln!("Create stderr file");
    fs::write(tests_paths.error.clone(), "").map_err(|_| AdaJudgeError::InvalidProblem)?;

    Ok(())
}

fn run_solution(
    config: &ProblemConfig,
    input_path: &Path,
    tests_paths: &TestsPaths,
) -> Result<AdaJudgeVerdict, AdaJudgeError> {
    eprintln!("Open stdin file");
    let stdin_file = File::open(input_path).map_err(|e| {
        eprintln!("{e}");
        AdaJudgeError::InvalidProblem
    })?;
    eprintln!("Open stdout file");
    let stdout_file = File::create(tests_paths.output.clone()).map_err(|e| {
        eprintln!("{e}");
        AdaJudgeError::InvalidProblem
    })?;
    eprintln!("Open stderr file");
    let stderr_file = File::create(tests_paths.error.clone()).map_err(|e| {
        eprintln!("{e}");
        AdaJudgeError::InvalidProblem
    })?;

    eprintln!("Run solution cmd");
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
            &format!("{}:/sandbox/bin:ro", tests_paths.solution.display()),
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
            eprintln!("{e}");
            AdaJudgeError::Bug
        })?;

    let timeout = Duration::from_millis(config.limits.time_limit_ms);
    let solution_status = solution_cmd.wait_timeout(timeout).map_err(|e| {
        eprintln!("{e}");
        AdaJudgeError::Bug
    })?;

    _ = solution_cmd.kill();
    eprintln!("Check solution status");
    match solution_status {
        None => Ok(AdaJudgeVerdict::TimeLimitExceeded),
        Some(status) => match status.code() {
            Some(VERDICT_OK) => Ok(AdaJudgeVerdict::Ok),
            Some(VERDICT_MLE) => Ok(AdaJudgeVerdict::MemoryLimitExceeded),
            _ => Ok(AdaJudgeVerdict::RuntimeError),
        },
    }
}

fn run_checker(
    config: &ProblemConfig,
    input_path: &Path,
    answer_path: PathBuf,
    tests_paths: &TestsPaths,
) -> Result<CheckerResult, AdaJudgeError> {
    eprintln!("Open stderr file");

    let stderr_file = File::create(tests_paths.error.clone()).map_err(|e| {
        eprintln!("{e}");
        AdaJudgeError::InvalidProblem
    })?;

    eprintln!("Run checker cmd");
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
            &format!("{}:/sandbox/bin:ro", tests_paths.checker.display()),
            "-v",
            &format!("{}:/sandbox/input:ro", input_path.display()),
            "-v",
            &format!("{}:/sandbox/output:ro", tests_paths.output.display()),
            "-v",
            &format!("{}:/sandbox/answer:ro", answer_path.display()),
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
            eprintln!("{e}");
            AdaJudgeError::Bug
        })?;

    let timeout = Duration::from_millis(config.limits.time_limit_ms);
    let checker_status = checker_cmd.wait_timeout(timeout).map_err(|e| {
        eprintln!("{e}");
        AdaJudgeError::Bug
    })?;

    _ = checker_cmd.kill();
    eprintln!("Check checker status");
    match checker_status {
        None => Err(AdaJudgeError::CheckerFailed),
        Some(status) => {
            let checker_msg = fs::read_to_string(tests_paths.error.clone()).map_err(|e| {
                eprintln!("{e}");
                AdaJudgeError::InvalidProblem
            })?;

            match status.code() {
                Some(CHECKER_OK) => Ok(CheckerResult {
                    verdict: AdaJudgeVerdict::Ok,
                    checker_msg,
                }),
                Some(CHECKER_WA) => Ok(CheckerResult {
                    verdict: AdaJudgeVerdict::WrongAnswer,
                    checker_msg,
                }),
                Some(CHECKER_PE) => Ok(CheckerResult {
                    verdict: AdaJudgeVerdict::PresentationError,
                    checker_msg,
                }),
                _ => Err(AdaJudgeError::CheckerFailed),
            }
        }
    }
}

fn run_single_test(
    config: &ProblemConfig,
    tests_paths: &TestsPaths,
    test_id: u8,
) -> Result<CheckerResult, AdaJudgeError> {
    let test_path = tests_paths.tests.join(test_id.to_string());

    let input_path = test_path.join("in");
    let answer_path = test_path.join("out");

    eprintln!("Run solution");
    let solution_verdict = run_solution(config, &input_path, tests_paths)?;

    if solution_verdict != AdaJudgeVerdict::Ok {
        eprintln!("Run result isn't OK");
        return Ok(CheckerResult {
            verdict: solution_verdict,
            checker_msg: String::default(),
        });
    }

    eprintln!("Run checker");
    run_checker(config, &input_path, answer_path, tests_paths)
}

pub async fn test(submission: SubmissionTask, pool: Data<PgPool>) -> Result<(), BoxDynError> {
    let id = submission.id;

    eprintln!("Test submission #{id}");

    eprintln!("Update total verdict");
    sqlx::query("update submissions set total_verdict = $1 where id = $2")
        .bind(AdaJudgeTotalVerdict::Testing)
        .bind(id)
        .execute(&*pool)
        .await
        .map_err(|e| {
            eprintln!("{e}");
            AdaJudgeError::Bug
        })?;

    eprintln!("Canonicalize problem path");
    let problem_path = submission.problem_path.canonicalize().map_err(|e| {
        eprintln!("{e}");
        AdaJudgeError::InvalidProblem
    })?;

    eprintln!("Canonicalize problem path");
    let run_path = submission.run_path.canonicalize().map_err(|e| {
        eprintln!("{e}");
        AdaJudgeError::InvalidProblem
    })?;

    eprintln!("Load problem's config");
    let config: ProblemConfig = toml::from_str(
        &read_to_string(problem_path.join("config.toml")).map_err(|e| {
            eprintln!("{e}");
            AdaJudgeError::InvalidProblem
        })?,
    )
    .map_err(|e| {
        eprintln!("{e}");
        AdaJudgeError::InvalidProblem
    })?;

    eprintln!("Check subgroups' for correctness");
    for (i, group) in config.test_groups.iter().enumerate() {
        eprintln!("Check subroup #{i} for correctness");
        if let Some(depends_on) = group.depends_on.clone() {
            for x in depends_on {
                if x >= i {
                    eprintln!("Subgroup depends on a subgroup, which has index less than its");
                    return Err(AdaJudgeError::InvalidProblem.into());
                }
            }
        }
    }

    eprintln!("Create tests paths");
    let tests_paths = TestsPaths::new(&run_path);

    eprintln!("Prepare test env");
    prepare_test_env(problem_path, &config, &tests_paths)?;

    let mut total_score = 0;
    let mut groups_result: Vec<GroupResult> = Vec::with_capacity(config.test_groups.len());

    dbg!("Test solution on subgroups");
    for test_group in config.test_groups.clone() {
        eprintln!("Test on next subgroup");
        eprintln!("Insert a subgroup's testing result");

        let result_id: i64 = sqlx::query_scalar(
            "insert into submissions_subgroups_results (subgroup_id, submission_id, verdict, score, checker_msg) values ($1, $2, $3, $4, $5)",
        )
        .bind(groups_result.len() as i64)
        .bind(id)
        .bind(AdaJudgeVerdict::Testing)
        .bind(0)
        .bind("")
        .fetch_one(&*pool)
        .await
        .map_err(|e| {
            eprintln!("{e}");
            AdaJudgeError::Bug
        })?;

        let mut test_result = GroupResult {
            verdict: AdaJudgeVerdict::Ok,
            test: 0,
            score: test_group.score,
            checker_msg: String::new(),
        };

        eprintln!("Check subgroup's dependencies");
        if let Some(depends_on) = test_group.depends_on {
            for i in depends_on {
                if groups_result[i].verdict != AdaJudgeVerdict::Ok {
                    eprintln!("Subgroup's dependency isn't OK, skip testing");
                    test_result.verdict = AdaJudgeVerdict::Skipped;
                    test_result.score = 0;
                    break;
                }
            }
        }

        if test_result.verdict != AdaJudgeVerdict::Skipped {
            eprintln!("Test solution on tests");
            for test_id in test_group.tests {
                eprintln!("Run test #{test_id}");

                let run_result = run_single_test(&config, &tests_paths, test_id)?;

                test_result.verdict = run_result.verdict.clone();
                test_result.test = test_id;
                test_result.checker_msg = run_result.checker_msg;

                if run_result.verdict != AdaJudgeVerdict::Ok {
                    eprintln!("Verdict isn't OK, skip testing");
                    test_result.score = 0;
                    break;
                }
            }
        }

        total_score += test_result.score;
        groups_result.push(test_result.clone());

        eprintln!("Update subgroup's test result record");
        sqlx::query(
            "update submissions_subgroups_results set verdict = $1, score = $2, checker_msg = $3 where id = $4",
        )
        .bind(test_result.verdict)
        .bind(test_result.score as i64)
        .bind(test_result.checker_msg)
        .bind(result_id)
        .execute(&*pool)
        .await
        .map_err(|e| {
            eprintln!("{e}");
            AdaJudgeError::Bug
        })?;
    }

    eprintln!("Update total test result");
    sqlx::query("update submissions set total_verdict = $1, total_score = $2 WHERE id = $3")
        .bind(match total_score {
            100 => AdaJudgeTotalVerdict::Ok,
            _ => AdaJudgeTotalVerdict::PartialSolution,
        })
        .bind(total_score as i64)
        .bind(id)
        .execute(&*pool)
        .await
        .map_err(|e| {
            eprintln!("{e}");
            AdaJudgeError::Bug
        })?;

    Ok(())
}

pub async fn push_submission_into_queue(
    State(state): State<Arc<AppState>>,
    Json(submission): Json<Submission>,
) -> Result<Json<i64>, Json<AdaJudgeError>> {
    // TODO: replace id with real user id and problem path to problem id

    eprintln!("Push to queue: {submission:?}");

    let id: i64 = sqlx::query_scalar(
        "insert into submissions (problem_id, user_id, total_verdict, total_score) values ($1, $2, $3, $4) returning id",
    )
    .bind(
        submission
            .problem_path
            .to_str()
            .ok_or(AdaJudgeError::InvalidProblem)?,
    )
    .bind(100)
    .bind(AdaJudgeTotalVerdict::Pending)
    .bind(0)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        eprintln!("{e}");
        AdaJudgeError::Bug
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
            eprintln!("{e}");
            AdaJudgeError::Bug
        })?;

    Ok(Json(id))
}
