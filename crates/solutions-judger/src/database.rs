use models::verdicts::{SubgroupVerdict, TotalVerdict};
use sqlx::PgPool;
use std::path::Path;

use crate::tools::MapLogExt;

pub async fn insert_submission(pool: &PgPool, problem_id: &Path) -> Result<i64, TotalVerdict> {
    let submission_id = sqlx::query_scalar(
        "insert into submissions (problem_id, user_id, total_verdict, total_score) values ($1, $2, $3, $4) returning id",
    )
        .bind(
            problem_id.display().to_string(),
        )
        .bind(100)
        .bind(TotalVerdict::Pending)
        .bind(0)
        .fetch_one(pool)
        .await
        .map_log(TotalVerdict::Bug)?;
    Ok(submission_id)
}

pub async fn update_total_testing_result(
    pool: &PgPool,
    submission_id: i64,
    verdict: &TotalVerdict,
    score: i32,
) -> Result<(), TotalVerdict> {
    sqlx::query("update submissions set total_verdict = $1, total_score = $2 where id = $3")
        .bind(verdict)
        .bind(score)
        .bind(submission_id)
        .execute(pool)
        .await
        .map_log(TotalVerdict::Bug)?;
    Ok(())
}

pub async fn insert_subgroup_testing_result(
    pool: &PgPool,
    subgroup_id: i64,
    submission_id: i64,
) -> Result<i64, TotalVerdict> {
    let subgroup_testing_result_id = sqlx::query_scalar(
        "insert into submissions_subgroups_results (subgroup_id, submission_id, verdict, test, score, checker_msg) values ($1, $2, $3, $4, $5, $6) returning id"
    )
        .bind(subgroup_id)
        .bind(submission_id)
        .bind(SubgroupVerdict::Testing)
        .bind(0)
        .bind(0)
        .bind("")
        .fetch_one(pool)
        .await
        .map_log(TotalVerdict::Bug)?;
    Ok(subgroup_testing_result_id)
}

pub async fn update_subgroup_testing_result(
    pool: &PgPool,
    subgroup_testing_result_id: i64,
    verdict: &SubgroupVerdict,
    test: i32,
    score: i32,
    checker_msg: String,
) -> Result<(), TotalVerdict> {
    sqlx::query(
        "update submissions_subgroups_results set verdict = $1, test = $2, score = $3, checker_msg = $4 where id = $5",
    )
        .bind(verdict)
        .bind(test)
        .bind(score)
        .bind(checker_msg)
        .bind(subgroup_testing_result_id)
        .execute(pool)
        .await
        .map_log(TotalVerdict::Bug)?;
    Ok(())
}
