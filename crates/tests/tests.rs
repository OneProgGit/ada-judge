use apalis::prelude::Data;
use models::{
    testing::{GroupResult, SubmissionTask, TotalResult},
    verdicts::{SubgroupVerdict, TotalVerdict},
};
use sqlx::{FromRow, PgPool, postgres::PgRow};
use tokio::fs;

async fn test_usual(
    pool: &PgPool,
    solution_name: &str,
    with_deps: bool,
    total_verdict: TotalVerdict,
    verdict: SubgroupVerdict,
) {
    let env_path = fs::canonicalize(if with_deps {
        format!("env_{}_with_deps", solution_name)
    } else {
        format!("env_{}_no_deps", solution_name)
    })
    .await
    .unwrap();

    fs::copy(
        &format!("solutions/{}.rs", solution_name),
        env_path.join("run.rs"),
    )
    .await
    .unwrap();

    let problem_id = if with_deps { 2 } else { 1 };

    let mut submission = SubmissionTask {
        problem_path: format!("problems/{problem_id}").into(),
        run_dir: env_path.clone(),
        id: 0,
    };
    let id: i64 = sqlx::query_scalar("insert into submissions (problem_id, user_id, total_verdict, total_score) values ($1, $2, $3, $4) returning id")
        .bind(submission.problem_path.to_str().unwrap())
        .bind(123)
        .bind(TotalVerdict::Pending)
        .bind(0).fetch_one(pool).await.unwrap();

    submission.id = id;

    solutions_judger::test(submission, Data::new(pool.clone()))
        .await
        .unwrap();

    fs::remove_dir_all(&env_path).await.unwrap();
    fs::create_dir(&env_path).await.unwrap();

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

    if verdict != SubgroupVerdict::Ok {
        assert_eq!(total_result.total_score, 0);
        assert_eq!(subgroups_results[1].score, 0);
        if with_deps {
            assert_eq!(subgroups_results[1].verdict, SubgroupVerdict::Skipped);
        } else {
            assert_eq!(subgroups_results[1].verdict, verdict);
        }
    } else {
        assert_eq!(total_result.total_score, 100);
        assert_eq!(subgroups_results[1].score, 100);
        assert_eq!(subgroups_results[1].verdict, verdict);
    }
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_ok_no_deps(pool: PgPool) {
    test_usual(&pool, "ok", false, TotalVerdict::Ok, SubgroupVerdict::Ok).await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_wa_no_deps(pool: PgPool) {
    test_usual(
        &pool,
        "wa",
        false,
        TotalVerdict::PartialSolution,
        SubgroupVerdict::WrongAnswer,
    )
    .await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_tle_no_deps(pool: PgPool) {
    test_usual(
        &pool,
        "tle",
        false,
        TotalVerdict::PartialSolution,
        SubgroupVerdict::TimeLimitExceeded,
    )
    .await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_mle_no_deps(pool: PgPool) {
    test_usual(
        &pool,
        "mle",
        false,
        TotalVerdict::PartialSolution,
        SubgroupVerdict::MemoryLimitExceeded,
    )
    .await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_re_no_deps(pool: PgPool) {
    test_usual(
        &pool,
        "re",
        false,
        TotalVerdict::PartialSolution,
        SubgroupVerdict::RuntimeError,
    )
    .await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_ok_with_deps(pool: PgPool) {
    test_usual(&pool, "ok", true, TotalVerdict::Ok, SubgroupVerdict::Ok).await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_wa_with_deps(pool: PgPool) {
    test_usual(
        &pool,
        "wa",
        true,
        TotalVerdict::PartialSolution,
        SubgroupVerdict::WrongAnswer,
    )
    .await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_tle_with_deps(pool: PgPool) {
    test_usual(
        &pool,
        "tle",
        true,
        TotalVerdict::PartialSolution,
        SubgroupVerdict::TimeLimitExceeded,
    )
    .await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_mle_with_deps(pool: PgPool) {
    test_usual(
        &pool,
        "mle",
        true,
        TotalVerdict::PartialSolution,
        SubgroupVerdict::MemoryLimitExceeded,
    )
    .await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_re_with_deps(pool: PgPool) {
    test_usual(
        &pool,
        "re",
        true,
        TotalVerdict::PartialSolution,
        SubgroupVerdict::RuntimeError,
    )
    .await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_incorrect_deps(pool: PgPool) {
    let env_path = fs::canonicalize("env_incorrect_deps").await.unwrap();

    fs::copy("solutions/ok.rs", env_path.join("run.rs"))
        .await
        .unwrap();

    let mut submission = SubmissionTask {
        problem_path: "problems/3".into(),
        run_dir: env_path.clone(),
        id: 0,
    };
    let id: i64 = sqlx::query_scalar("insert into submissions (problem_id, user_id, total_verdict, total_score) values ($1, $2, $3, $4) returning id")
        .bind(submission.problem_path.to_str().unwrap())
        .bind(123)
        .bind(TotalVerdict::Pending)
        .bind(0)
        .fetch_one(&pool)
        .await
        .unwrap();

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
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(total_result.total_score, 0);
    assert_eq!(total_result.total_verdict, TotalVerdict::InvalidProblem);

    fs::remove_dir_all(&env_path).await.unwrap();
    fs::create_dir(&env_path).await.unwrap();
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_ce(pool: PgPool) {
    let env_path = fs::canonicalize("env_ce").await.unwrap();

    fs::copy("solutions/ce.rs", env_path.join("run.rs"))
        .await
        .unwrap();

    let mut submission = SubmissionTask {
        problem_path: "problems/1".into(),
        run_dir: env_path.clone(),
        id: 0,
    };
    let id: i64 = sqlx::query_scalar("insert into submissions (problem_id, user_id, total_verdict, total_score) values ($1, $2, $3, $4) returning id")
        .bind(submission.problem_path.to_str().unwrap())
        .bind(123)
        .bind(TotalVerdict::Pending)
        .bind(0)
        .fetch_one(&pool)
        .await
        .unwrap();

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
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(total_result.total_score, 0);
    assert_eq!(total_result.total_verdict, TotalVerdict::CompilationError);

    fs::remove_dir_all(&env_path).await.unwrap();
    fs::create_dir(&env_path).await.unwrap();
}
