//! Database tools for `ada-judge`

#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![deny(warnings)]
#![deny(missing_docs)]
#![deny(rustdoc::all)]
#![deny(rustdoc::broken_intra_doc_links)]
#![forbid(unsafe_code)]

use models::{
    problem_config::{DatabaseProblemConfig, Subgroup},
    verdicts::{SubgroupVerdict, TotalVerdict},
};
use sqlx::{PgPool, types::Json};
use tools::map::MapLogExt;

/// Creates a user with login and password hash and returns it's id
/// # Errors
/// Returns an error if the user with this login exists
pub async fn create_user(
    pool: &PgPool,
    login: &str,
    password_hash: &str,
) -> Result<i64, TotalVerdict> {
    let user_id = sqlx::query_scalar!(
        r"insert into users (login, password_hash) values ($1, $2) returning id",
        login,
        password_hash
    )
    .fetch_one(pool)
    .await
    .map_log(TotalVerdict::InvalidRequest)?;

    Ok(user_id)
}

/// Get's problem's config from `problems` table
/// # Errors
/// Returns an error if `problem_id` is invalid
pub async fn get_problem_config(
    pool: &PgPool,
    problem_id: i64,
) -> Result<DatabaseProblemConfig, TotalVerdict> {
    let config = sqlx::query_as!(
        DatabaseProblemConfig,
        r#"select
                c.name,
                c.time_limit_ms,
                c.memory_limit_mb,
                c.checker_path,
                c.tests_path,
                coalesce(
                    json_agg(
                        json_build_object(
                            'type', v.type,
                            'tests', v.tests,
                            'score', v.score,
                            'depends_on', v.depends_on
                        ) order by v.subgroup_index
                    ), '[]'::json
                ) as "subgroups!: Json<Vec<Subgroup>>"
            from problems c
            left join problems_subgroups v on v.problem_id = c.id
            where c.id = $1
            group by c.id, c.name, c.time_limit_ms, c.memory_limit_mb, c.checker_path, c.tests_path;
        "#,
        problem_id
    )
    .fetch_one(pool)
    .await
    .map_log(TotalVerdict::InvalidRequest)?;

    Ok(config)
}

/// Inserts a submission to `submissions` table and returns it's id
/// # Errors
/// Returns an error if `problem_id` is invalid
pub async fn insert_submission(pool: &PgPool, problem_id: i64) -> Result<i64, TotalVerdict> {
    let submission_id = sqlx::query_scalar!(
        r"insert into submissions (problem_id, user_id, total_verdict, total_score) 
          values ($1, $2, $3::total_verdict, $4) returning id",
        problem_id,
        None::<i64>,
        TotalVerdict::Pending as _,
        0
    )
    .fetch_one(pool)
    .await
    .map_log(TotalVerdict::InvalidRequest)?;

    Ok(submission_id)
}

/// Updates total testing result for a submission
/// # Errors
/// Returns an error if `submission_id` is invalid
pub async fn update_total_testing_result(
    pool: &PgPool,
    submission_id: i64,
    verdict: &TotalVerdict,
    score: i32,
) -> Result<(), TotalVerdict> {
    sqlx::query!(
        r"update submissions set total_verdict = $1::total_verdict, total_score = $2 
            where id = $3",
        verdict as _,
        score,
        submission_id
    )
    .execute(pool)
    .await
    .map_log(TotalVerdict::InvalidRequest)?;
    Ok(())
}

/// Inserts a subgroup's testing result to `submissions_subgroups_results` table
/// # Errors
/// Returns an error if `submission_id` is invalid. `TODO`: return an error if `subgroup_id` is out of range
pub async fn insert_subgroup_testing_result(
    pool: &PgPool,
    submission_id: i64,
    subgroup_id: i32,
) -> Result<(), TotalVerdict> {
    sqlx::query!(
        r"insert into submissions_subgroups_results (subgroup_id, submission_id, subgroup_verdict, test, score, checker_msg)
            values ($1, $2, $3::subgroup_verdict, $4, $5, $6)",
        subgroup_id,
        submission_id,
        SubgroupVerdict::Testing as _,
        0,
        0,
        ""
    )
        .execute(pool)
        .await
        .map_log(TotalVerdict::InvalidRequest)?;
    Ok(())
}

/// Updates testing result for a subgroup of the problem
/// # Errors
/// Returns an error if `submission_id` is invalid. `TODO`: return an error if `subgroup_id` is out of range
pub async fn update_subgroup_testing_result(
    pool: &PgPool,
    submission_id: i64,
    subgroup_id: i32,
    verdict: &SubgroupVerdict,
    test: i32,
    score: i32,
    checker_msg: String,
) -> Result<(), TotalVerdict> {
    sqlx::query!(
        r"update submissions_subgroups_results set subgroup_verdict = $1::subgroup_verdict, test = $2, score = $3, checker_msg = $4 
            where submission_id = $5 and subgroup_id = $6",
        verdict as _,
        test,
        score,
        checker_msg,
        submission_id,
        subgroup_id
    )
        .execute(pool)
        .await
        .map_log(TotalVerdict::InvalidRequest)?;
    Ok(())
}
