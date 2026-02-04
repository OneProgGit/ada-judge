//! Judgement system made with Rust.

use std::{
    fs::{self, File, read_to_string},
    path::PathBuf,
    process::{Command, Stdio},
    time::Duration,
};

use fs_extra::dir::CopyOptions;
use wait_timeout::ChildExt;

use crate::{problem_config::ProblemConfig, verdicts::Verdict};

pub mod problem_config;
pub mod verdicts;

const CHECKER_OK: i32 = 0;
const CHECKER_WA: i32 = 1;
const CHECKER_PE: i32 = 2;

/// Test solution and return a verdict for each subgroup.
pub fn test(problem_path: PathBuf, run_path: PathBuf) -> Result<Vec<(Verdict, u8)>, Verdict> {
    let problem: ProblemConfig = toml::from_str(
        &read_to_string(problem_path.join("config.toml"))
            .map_err(|err| Verdict::InvalidProblem(format!("Invalid config path: {err}")))?,
    )
    .map_err(|err| Verdict::InvalidProblem(format!("Invalid config: {err}")))?;

    fs::copy(
        problem_path.join(problem.checker.path),
        run_path.join("checker"),
    )
    .map_err(|err| Verdict::InvalidProblem(format!("Invalid checker path: {err}")))?;

    let mut opt = CopyOptions::new();
    opt.overwrite = true;
    opt.copy_inside = true;
    opt.content_only = false;

    fs_extra::dir::copy(
        problem_path.join(problem.tests.path),
        run_path.join("tests"),
        &opt,
    )
    .map_err(|err| Verdict::InvalidProblem(format!("Invalid tests path: {err}")))?;

    fs::write(run_path.join("stderr.txt"), "")
        .map_err(|err| Verdict::Fail(format!("Failed to create `stderr.txt`: {err}")))?;

    let mut res: Vec<(Verdict, u8)> = Vec::with_capacity(problem.test_groups.len());

    for test_group in problem.test_groups {
        let (mut verdict, mut test) = (Verdict::Ok, 0);

        for test_id in test_group.tests {
            let test_path = run_path.join("tests").join(test_id.to_string());

            let input_path = test_path.join("in");
            let output_path = run_path.join("stdout.txt");
            let answer_path = test_path.join("out");

            let solution_path = run_path.join("run");
            let checker_path = run_path.join("checker");

            let stdin_file = File::open(input_path.clone()).map_err(|err| {
                Verdict::InvalidProblem(format!("Invalid test format on test `{}`: {err}", test_id))
            })?;
            let stdout_file = File::create(output_path.clone()).map_err(|err| {
                Verdict::InvalidProblem(format!("Invalid test format on test `{}`: {err}", test_id))
            })?;

            let mut solution_cmd = Command::new(solution_path)
                .stdin(Stdio::from(stdin_file))
                .stdout(Stdio::from(stdout_file))
                .spawn()
                .map_err(|err| {
                    Verdict::Fail(format!("Failed to test solution on test `{test}`: {err}"))
                })?;

            let timeout = Duration::from_millis(problem.limits.time_limit_ms);
            let solution_status = solution_cmd.wait_timeout(timeout).map_err(|err| {
                Verdict::Fail(format!("Failed to test solution on test `{test}`: {err}"))
            })?;

            _ = solution_cmd.kill();
            match solution_status {
                None => {
                    verdict = Verdict::TimeLimitExceeded;
                    test = test_id;
                    break;
                }
                Some(status) => match status.code() {
                    Some(0) => {
                        let mut checker_cmd = Command::new(checker_path)
                            .args([
                                input_path.as_os_str(),
                                output_path.as_os_str(),
                                answer_path.as_os_str(),
                            ])
                            .spawn()
                            .map_err(|err| {
                                Verdict::Fail(format!(
                                    "Failed to check solution on test `{test}`: {err}"
                                ))
                            })?;

                        let timeout = Duration::from_millis(problem.limits.time_limit_ms);
                        let checker_status = checker_cmd.wait_timeout(timeout).map_err(|err| {
                            Verdict::Fail(format!(
                                "Failed to check solution on test `{test}`: {err}"
                            ))
                        })?;

                        _ = checker_cmd.kill();
                        match checker_status {
                            None => {
                                return Err(Verdict::Fail(format!(
                                    "Failed to check solution on test `{test}`: waiting checker for too long"
                                )));
                            }
                            Some(status) => match status.code() {
                                Some(CHECKER_OK) => {}
                                Some(CHECKER_WA) => {
                                    verdict = Verdict::WrongAnswer;
                                    test = test_id;
                                    break;
                                }
                                Some(CHECKER_PE) => {
                                    verdict = Verdict::PresentationError;
                                    test = test_id;
                                    break;
                                }
                                _ => {
                                    return Err(Verdict::Fail(format!(
                                        "Failed to check solution on test `{test}`: checker failed"
                                    )));
                                }
                            },
                        }
                    }
                    _ => {
                        verdict = Verdict::RuntimeError;
                        test = test_id;
                        break;
                    }
                },
            }
        }

        res.push((verdict, test));
    }

    Ok(res)
}
