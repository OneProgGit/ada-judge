#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![deny(warnings)]
#![forbid(unsafe_code)]

use apalis::prelude::Data;
use models::testing::get_lang_str;
use models::{
    testing::{Language, SubgroupResult, SubmissionTask, TotalResult},
    verdicts::{SubgroupVerdict, TotalVerdict},
};
use sqlx::{FromRow, PgPool, postgres::PgRow};
use tempfile::tempdir;
use tokio::fs;

async fn test_1(
    pool: &PgPool,
    solution_name: &str,
    total_verdict: TotalVerdict,
    verdict: SubgroupVerdict,
    lang: Language,
) {
    let env_path = tempdir().unwrap();
    let lang_str = get_lang_str(&lang);

    fs::copy(
        &format!("solutions/{lang_str}/{solution_name}.{lang_str}"),
        env_path.path().join(format!("run.{lang_str}")),
    )
    .await
    .unwrap();

    let problem_id = 1;

    let mut submission = SubmissionTask {
        problem_path: format!("../../problems/{problem_id}").into(),
        problem_id,
        run_dir: env_path.path().to_path_buf(),
        lang,
        id: 0,
    };
    let id: i64 = sqlx::query_scalar("insert into submissions (problem_id, user_id, total_verdict, total_score) values ($1, $2, $3, $4) returning id")
        .bind(submission.problem_id)
        .bind(None::<i64>)
        .bind(TotalVerdict::Pending)
        .bind(0).fetch_one(pool).await.unwrap();

    submission.id = id;

    submissions_judger::test_submission(submission, Data::new(pool.clone()))
        .await
        .unwrap();

    let total_result: TotalResult = sqlx::query(
        "select total_verdict, total_score 
             from submissions where id = $1",
    )
    .bind(id)
    .map(|row: PgRow| TotalResult::from_row(&row).unwrap())
    .fetch_one(pool)
    .await
    .unwrap();

    let subgroups_results: Vec<SubgroupResult> = sqlx::query(
        "select subgroup_verdict, score, test, score, checker_msg
             from submissions_subgroups_results
             where submission_id = $1
             order by subgroup_index",
    )
    .bind(id)
    .map(|row: PgRow| SubgroupResult::from_row(&row).unwrap())
    .fetch_all(pool)
    .await
    .unwrap();

    assert_eq!(total_result.total_verdict, total_verdict);
    assert_eq!(subgroups_results[0].score, 0);
    assert_eq!(subgroups_results[0].subgroup_verdict, verdict);

    if verdict == SubgroupVerdict::Ok {
        assert_eq!(total_result.total_score, 100);
        assert_eq!(subgroups_results[1].score, 100);
    } else {
        assert_eq!(total_result.total_score, 0);
        assert_eq!(subgroups_results[1].score, 0);
    }
    assert_eq!(subgroups_results[1].subgroup_verdict, verdict);
}

async fn test_2(
    pool: &PgPool,
    solution_name: &str,
    total_verdict: TotalVerdict,
    verdict: SubgroupVerdict,
    lang: Language,
) {
    let env_path = tempdir().unwrap();
    let lang_str = get_lang_str(&lang);

    fs::copy(
        &format!("solutions/{lang_str}/{solution_name}.{lang_str}"),
        env_path.path().join(format!("run.{lang_str}")),
    )
    .await
    .unwrap();

    let problem_id = 2;

    let mut submission = SubmissionTask {
        problem_path: format!("../../problems/{problem_id}").into(),
        problem_id,
        run_dir: env_path.path().to_path_buf(),
        lang,
        id: 0,
    };
    let id: i64 = sqlx::query_scalar("insert into submissions (problem_id, user_id, total_verdict, total_score) values ($1, $2, $3, $4) returning id")
        .bind(submission.problem_id)
        .bind(None::<i64>)
        .bind(TotalVerdict::Pending)
        .bind(0).fetch_one(pool).await.unwrap();

    submission.id = id;

    submissions_judger::test_submission(submission, Data::new(pool.clone()))
        .await
        .unwrap();

    let total_result: TotalResult = sqlx::query(
        "select total_verdict, total_score 
             from submissions where id = $1",
    )
    .bind(id)
    .map(|row: PgRow| TotalResult::from_row(&row).unwrap())
    .fetch_one(pool)
    .await
    .unwrap();

    let subgroups_results: Vec<SubgroupResult> = sqlx::query(
        "select subgroup_verdict, score, test, score, checker_msg
             from submissions_subgroups_results
             where submission_id = $1
             order by subgroup_index",
    )
    .bind(id)
    .map(|row: PgRow| SubgroupResult::from_row(&row).unwrap())
    .fetch_all(pool)
    .await
    .unwrap();

    assert_eq!(total_result.total_verdict, total_verdict);
    assert_eq!(subgroups_results[0].score, 0);
    assert_eq!(subgroups_results[0].subgroup_verdict, verdict);

    if verdict == SubgroupVerdict::Ok {
        assert_eq!(total_result.total_score, 100);
        assert_eq!(subgroups_results[1].score, 50);
        assert_eq!(subgroups_results[2].score, 50);
        assert_eq!(subgroups_results[1].subgroup_verdict, verdict);
    } else {
        assert_eq!(total_result.total_score, 0);
        assert_eq!(subgroups_results[1].score, 0);
        assert_eq!(subgroups_results[2].score, 0);
        assert_eq!(
            subgroups_results[1].subgroup_verdict,
            SubgroupVerdict::Skipped
        );
    }
    assert_eq!(subgroups_results[2].subgroup_verdict, verdict);
}

async fn test_3(pool: &PgPool, lang: Language) {
    let env_path = tempdir().unwrap();
    let lang_str = get_lang_str(&lang);

    fs::copy(
        format!("solutions/{lang_str}/ok.{lang_str}"),
        env_path.path().join(format!("run.{lang_str}")),
    )
    .await
    .unwrap();

    let mut submission = SubmissionTask {
        problem_path: "../../problems/3".into(),
        problem_id: 3,
        run_dir: env_path.path().to_path_buf(),
        lang,
        id: 0,
    };
    let id: i64 = sqlx::query_scalar("insert into submissions (problem_id, user_id, total_verdict, total_score) values ($1, $2, $3, $4) returning id")
        .bind(submission.problem_id)
        .bind(None::<i64>)
        .bind(TotalVerdict::Pending)
        .bind(0)
        .fetch_one(pool)
        .await
        .unwrap();

    submission.id = id;

    submissions_judger::test_submission(submission, Data::new(pool.clone()))
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
    assert_eq!(total_result.total_verdict, TotalVerdict::InvalidProblem);
}

async fn test_ce_common(pool: &PgPool, lang: Language) {
    let env_path = tempdir().unwrap();
    let lang_str = get_lang_str(&lang);

    fs::copy(
        format!("solutions/{lang_str}/ce.{lang_str}"),
        env_path.path().join(format!("run.{lang_str}")),
    )
    .await
    .unwrap();

    let mut submission = SubmissionTask {
        problem_path: "../../problems/1".into(),
        problem_id: 1,
        run_dir: env_path.path().to_path_buf(),
        lang,
        id: 0,
    };
    let id: i64 = sqlx::query_scalar("insert into submissions (problem_id, user_id, total_verdict, total_score) values ($1, $2, $3, $4) returning id")
        .bind(submission.problem_id)
        .bind(None::<i64>)
        .bind(TotalVerdict::Pending)
        .bind(0)
        .fetch_one(pool)
        .await
        .unwrap();

    submission.id = id;

    submissions_judger::test_submission(submission, Data::new(pool.clone()))
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
    assert_eq!(total_result.total_verdict, TotalVerdict::CompilationError);
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_ok_no_deps(pool: PgPool) {
    test_1(
        &pool,
        "ok",
        TotalVerdict::Ok,
        SubgroupVerdict::Ok,
        Language::Rust,
    )
    .await;
    test_1(
        &pool,
        "ok",
        TotalVerdict::Ok,
        SubgroupVerdict::Ok,
        Language::Clang,
    )
    .await;
    test_1(
        &pool,
        "ok",
        TotalVerdict::Ok,
        SubgroupVerdict::Ok,
        Language::Go,
    )
    .await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_wa_no_deps(pool: PgPool) {
    test_1(
        &pool,
        "wa",
        TotalVerdict::PartialSolution,
        SubgroupVerdict::WrongAnswer,
        Language::Rust,
    )
    .await;
    test_1(
        &pool,
        "wa",
        TotalVerdict::PartialSolution,
        SubgroupVerdict::WrongAnswer,
        Language::Clang,
    )
    .await;
    test_1(
        &pool,
        "wa",
        TotalVerdict::PartialSolution,
        SubgroupVerdict::WrongAnswer,
        Language::Go,
    )
    .await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_tle_no_deps(pool: PgPool) {
    test_1(
        &pool,
        "tle",
        TotalVerdict::PartialSolution,
        SubgroupVerdict::TimeLimitExceeded,
        Language::Rust,
    )
    .await;
    test_1(
        &pool,
        "tle",
        TotalVerdict::PartialSolution,
        SubgroupVerdict::TimeLimitExceeded,
        Language::Clang,
    )
    .await;
    test_1(
        &pool,
        "tle",
        TotalVerdict::PartialSolution,
        SubgroupVerdict::TimeLimitExceeded,
        Language::Go,
    )
    .await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_mle_no_deps(pool: PgPool) {
    test_1(
        &pool,
        "mle",
        TotalVerdict::PartialSolution,
        SubgroupVerdict::MemoryLimitExceeded,
        Language::Rust,
    )
    .await;
    test_1(
        &pool,
        "mle",
        TotalVerdict::PartialSolution,
        SubgroupVerdict::MemoryLimitExceeded,
        Language::Clang,
    )
    .await;
    test_1(
        &pool,
        "mle",
        TotalVerdict::PartialSolution,
        SubgroupVerdict::MemoryLimitExceeded,
        Language::Go,
    )
    .await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_re_no_deps(pool: PgPool) {
    test_1(
        &pool,
        "re",
        TotalVerdict::PartialSolution,
        SubgroupVerdict::RuntimeError,
        Language::Rust,
    )
    .await;
    test_1(
        &pool,
        "re",
        TotalVerdict::PartialSolution,
        SubgroupVerdict::RuntimeError,
        Language::Clang,
    )
    .await;
    test_1(
        &pool,
        "re",
        TotalVerdict::PartialSolution,
        SubgroupVerdict::RuntimeError,
        Language::Go,
    )
    .await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_ok_with_deps(pool: PgPool) {
    test_2(
        &pool,
        "ok",
        TotalVerdict::Ok,
        SubgroupVerdict::Ok,
        Language::Rust,
    )
    .await;
    test_2(
        &pool,
        "ok",
        TotalVerdict::Ok,
        SubgroupVerdict::Ok,
        Language::Clang,
    )
    .await;
    test_2(
        &pool,
        "ok",
        TotalVerdict::Ok,
        SubgroupVerdict::Ok,
        Language::Go,
    )
    .await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_wa_with_deps(pool: PgPool) {
    test_2(
        &pool,
        "wa",
        TotalVerdict::PartialSolution,
        SubgroupVerdict::WrongAnswer,
        Language::Rust,
    )
    .await;
    test_2(
        &pool,
        "wa",
        TotalVerdict::PartialSolution,
        SubgroupVerdict::WrongAnswer,
        Language::Clang,
    )
    .await;
    test_2(
        &pool,
        "wa",
        TotalVerdict::PartialSolution,
        SubgroupVerdict::WrongAnswer,
        Language::Go,
    )
    .await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_tle_with_deps(pool: PgPool) {
    test_2(
        &pool,
        "tle",
        TotalVerdict::PartialSolution,
        SubgroupVerdict::TimeLimitExceeded,
        Language::Rust,
    )
    .await;
    test_2(
        &pool,
        "tle",
        TotalVerdict::PartialSolution,
        SubgroupVerdict::TimeLimitExceeded,
        Language::Clang,
    )
    .await;
    test_2(
        &pool,
        "tle",
        TotalVerdict::PartialSolution,
        SubgroupVerdict::TimeLimitExceeded,
        Language::Go,
    )
    .await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_mle_with_deps(pool: PgPool) {
    test_2(
        &pool,
        "mle",
        TotalVerdict::PartialSolution,
        SubgroupVerdict::MemoryLimitExceeded,
        Language::Rust,
    )
    .await;
    test_2(
        &pool,
        "mle",
        TotalVerdict::PartialSolution,
        SubgroupVerdict::MemoryLimitExceeded,
        Language::Clang,
    )
    .await;
    test_2(
        &pool,
        "mle",
        TotalVerdict::PartialSolution,
        SubgroupVerdict::MemoryLimitExceeded,
        Language::Go,
    )
    .await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_re_with_deps(pool: PgPool) {
    test_2(
        &pool,
        "re",
        TotalVerdict::PartialSolution,
        SubgroupVerdict::RuntimeError,
        Language::Rust,
    )
    .await;
    test_2(
        &pool,
        "re",
        TotalVerdict::PartialSolution,
        SubgroupVerdict::RuntimeError,
        Language::Clang,
    )
    .await;
    test_2(
        &pool,
        "re",
        TotalVerdict::PartialSolution,
        SubgroupVerdict::RuntimeError,
        Language::Go,
    )
    .await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_incorrect_deps(pool: PgPool) {
    test_3(&pool, Language::Rust).await;
    test_3(&pool, Language::Clang).await;
    test_3(&pool, Language::Go).await;
}

#[sqlx::test(migrations = "../../migrations")]
async fn test_ce(pool: PgPool) {
    test_ce_common(&pool, Language::Rust).await;
    test_ce_common(&pool, Language::Clang).await;
    test_ce_common(&pool, Language::Go).await;
}
