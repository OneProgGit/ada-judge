use apalis::prelude::Data;
use models::{
    enums::{AdaJudgeTotalVerdict, AdaJudgeVerdict},
    testing::{GroupResult, SubmissionTask, TotalResult},
};
use sqlx::{FromRow, PgPool, postgres::PgRow};
use std::{fs, process::Command};

fn compile(solution_name: &str, env_path: String) {
    Command::new("rustc")
        .args([
            &format!("solutions/{}.rs", solution_name),
            "-o",
            &(env_path.clone() + "/run"),
        ])
        .spawn()
        .unwrap()
        .wait()
        .unwrap();
}

async fn test_usual(
    pool: &PgPool,
    solution_name: &str,
    with_deps: bool,
    total_verdict: AdaJudgeTotalVerdict,
    verdict: AdaJudgeVerdict,
) {
    let env_path = if with_deps {
        format!("env_{}_with_deps", solution_name)
    } else {
        format!("env_{}_no_deps", solution_name)
    };

    compile(solution_name, env_path.clone());

    let problem_id = if with_deps { 2 } else { 1 };

    let mut submission = SubmissionTask {
        problem_path: format!("problems/{problem_id}").into(),
        run_path: env_path.clone().into(),
        id: 0,
    };
    let id: i64 = sqlx::query_scalar("insert into submissions (problem_id, user_id, total_verdict, total_score) values ($1, $2, $3, $4) returning id")
        .bind(submission.problem_path.to_str().unwrap())
        .bind(123)
        .bind(AdaJudgeTotalVerdict::Pending)
        .bind(0).fetch_one(pool).await.unwrap();

    submission.id = id;

    solutions_judger::test(submission, Data::new(pool.clone()))
        .await
        .unwrap();

    fs::remove_dir_all(&env_path).unwrap();
    fs::create_dir(&env_path).unwrap();

    let total_result: TotalResult = sqlx::query(
        "select total_verdict, total_score 
             from submissions where id = $1",
    )
    .bind(id)
    .map(|row: PgRow| TotalResult::from_row(&row).unwrap())
    .fetch_one(pool)
    .await
    .unwrap();

    let subgroups_results: Vec<GroupResult> = sqlx::query(
        "select verdict, score, test, score, checker_msg
             from submissions_subgroups_results
             where submission_id = $1
             order by subgroup_id",
    )
    .bind(id)
    .map(|row: PgRow| GroupResult::from_row(&row).unwrap())
    .fetch_all(pool)
    .await
    .unwrap();

    assert_eq!(total_result.total_verdict, total_verdict);
    assert_eq!(subgroups_results[0].score, 0);
    assert_eq!(subgroups_results[0].verdict, verdict);

    if verdict != AdaJudgeVerdict::Ok {
        assert_eq!(total_result.total_score, 0);
        assert_eq!(subgroups_results[1].score, 0);
        if with_deps {
            assert_eq!(subgroups_results[1].verdict, AdaJudgeVerdict::Skipped);
        } else {
            assert_eq!(subgroups_results[1].verdict, verdict);
        }
    } else {
        assert_eq!(total_result.total_score, 100);
        assert_eq!(subgroups_results[1].score, 100);
        assert_eq!(subgroups_results[1].verdict, verdict);
    }
}

async fn test_incorrect_deps(pool: &PgPool, solution_name: &str) {
    let env_path = format!("env_{solution_name}_incorrect_deps");
    compile(solution_name, env_path.clone());
    let mut submission = SubmissionTask {
        problem_path: "problems/3".into(),
        run_path: env_path.clone().into(),
        id: 0,
    };
    let id: i64 = sqlx::query_scalar("insert into submissions (problem_id, user_id, total_verdict, total_score) values ($1, $2, $3, $4) returning id")
        .bind(submission.problem_path.to_str().unwrap())
        .bind(123)
        .bind(AdaJudgeTotalVerdict::Pending)
        .bind(0).fetch_one(pool).await.unwrap();

    submission.id = id;

    solutions_judger::test(submission, Data::new(pool.clone()))
        .await
        .unwrap_err();

    let total_result: TotalResult = sqlx::query(
        "select total_verdict, total_score 
             from submissions where id = $1",
    )
    .bind(id)
    .map(|row: PgRow| TotalResult::from_row(&row).unwrap())
    .fetch_one(pool)
    .await
    .unwrap();

    assert_eq!(total_result.total_score, 0);
    assert_eq!(
        total_result.total_verdict,
        AdaJudgeTotalVerdict::InvalidProblem
    );

    fs::remove_dir_all(&env_path).unwrap();
    fs::create_dir(&env_path).unwrap();
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_ok_no_deps(pool: PgPool) {
    test_usual(
        &pool,
        "ok",
        false,
        AdaJudgeTotalVerdict::Ok,
        AdaJudgeVerdict::Ok,
    )
    .await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_wa_no_deps(pool: PgPool) {
    test_usual(
        &pool,
        "wa",
        false,
        AdaJudgeTotalVerdict::PartialSolution,
        AdaJudgeVerdict::WrongAnswer,
    )
    .await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_tle_no_deps(pool: PgPool) {
    test_usual(
        &pool,
        "tle",
        false,
        AdaJudgeTotalVerdict::PartialSolution,
        AdaJudgeVerdict::TimeLimitExceeded,
    )
    .await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_mle_no_deps(pool: PgPool) {
    test_usual(
        &pool,
        "mle",
        false,
        AdaJudgeTotalVerdict::PartialSolution,
        AdaJudgeVerdict::MemoryLimitExceeded,
    )
    .await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_re_no_deps(pool: PgPool) {
    test_usual(
        &pool,
        "re",
        false,
        AdaJudgeTotalVerdict::PartialSolution,
        AdaJudgeVerdict::RuntimeError,
    )
    .await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_ok_with_deps(pool: PgPool) {
    test_usual(
        &pool,
        "ok",
        true,
        AdaJudgeTotalVerdict::Ok,
        AdaJudgeVerdict::Ok,
    )
    .await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_wa_with_deps(pool: PgPool) {
    test_usual(
        &pool,
        "wa",
        true,
        AdaJudgeTotalVerdict::PartialSolution,
        AdaJudgeVerdict::WrongAnswer,
    )
    .await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_tle_with_deps(pool: PgPool) {
    test_usual(
        &pool,
        "tle",
        true,
        AdaJudgeTotalVerdict::PartialSolution,
        AdaJudgeVerdict::TimeLimitExceeded,
    )
    .await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_mle_with_deps(pool: PgPool) {
    test_usual(
        &pool,
        "mle",
        true,
        AdaJudgeTotalVerdict::PartialSolution,
        AdaJudgeVerdict::MemoryLimitExceeded,
    )
    .await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_re_with_deps(pool: PgPool) {
    test_usual(
        &pool,
        "re",
        true,
        AdaJudgeTotalVerdict::PartialSolution,
        AdaJudgeVerdict::RuntimeError,
    )
    .await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_ok_incorrect_deps(pool: PgPool) {
    test_incorrect_deps(&pool, "ok").await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_wa_incorrect_deps(pool: PgPool) {
    test_incorrect_deps(&pool, "wa").await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_tle_incorrect_deps(pool: PgPool) {
    test_incorrect_deps(&pool, "tle").await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_mle_incorrect_deps(pool: PgPool) {
    test_incorrect_deps(&pool, "mle").await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_re_incorrect_deps(pool: PgPool) {
    test_incorrect_deps(&pool, "re").await;
}
