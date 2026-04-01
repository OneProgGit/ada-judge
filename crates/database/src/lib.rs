//! Database tools for `ada-judge`

#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![deny(clippy::nursery)]
#![deny(warnings)]
#![deny(missing_docs)]
#![deny(rustdoc::all)]
#![deny(rustdoc::broken_intra_doc_links)]
#![forbid(unsafe_code)]

use ::tools::map::MapLogExt;
use models::{
    contest_config::DatabaseContestConfig,
    contests::LeaderboardRow,
    problem_config::{DatabaseProblemConfig, PublicProblemConfig},
    testing::Submission,
    users::DatabaseUser,
    verdicts::{SubgroupVerdict, TotalVerdict},
};
use sqlx::PgPool;

pub mod tools;

/// Creates a user with login and password hash and returns it's id
/// # Errors
/// Returns an error if the user with this login exists
pub async fn create_user(
    pool: &PgPool,
    login: &str,
    password_hash: &str,
) -> Result<i64, TotalVerdict> {
    let user_id =
        sqlx::query_scalar("insert into users (login, password_hash) values ($1, $2) returning id")
            .bind(login)
            .bind(password_hash)
            .fetch_one(pool)
            .await
            .map_log(TotalVerdict::InvalidRequest)?;

    Ok(user_id)
}

/// Gets a user with target login
/// # Errors
/// Returns an error if the user with this login does not exist
pub async fn get_user_by_login(pool: &PgPool, login: &str) -> Result<DatabaseUser, TotalVerdict> {
    sqlx::query_as("select * from users where login = $1")
        .bind(login)
        .fetch_one(pool)
        .await
        .map_log(TotalVerdict::InvalidRequest)
}

/// Gets a user with target id
/// # Errors
/// Returns an error if the user with this id does not exist
pub async fn get_user_by_id(pool: &PgPool, id: i64) -> Result<DatabaseUser, TotalVerdict> {
    sqlx::query_as("select * from users where id = $1")
        .bind(id)
        .fetch_one(pool)
        .await
        .map_log(TotalVerdict::InvalidRequest)
}

/// Gets contest's leaderboard
/// # Errors
/// Returns an error if `contest_id` is invalid
pub async fn get_contest_leaderboard(
    pool: &PgPool,
    contest_id: i64,
) -> Result<Vec<LeaderboardRow>, TotalVerdict> {
    let leaderboard = sqlx::query_as(
        "with ranked as (
                select
                    s.user_id,
                    s.problem_id,
                    s.total_score,
                    row_number() over (
                        partition by s.user_id, s.problem_id
                        order by s.total_score desc
                    ) as rn
                from submissions s
                join problems p on p.id = s.problem_id
                where p.contest_id = $1
            ),
            best as (
                select user_id, problem_id, total_score
                from ranked
                where rn = 1
            ),
            users as (
                select distinct user_id
                from submissions s
                join problems p on p.id = s.problem_id
                where p.contest_id = $1
            ),
            contest_problems as (
                select id, problem_index
                from problems
                where contest_id = $1
            )
            select 
                u.user_id,
                array_agg(
                    coalesce(b.total_score, 0)
                    order by p.problem_index
                ) as scores,
                sum(coalesce(b.total_score, 0)) as total_score
            from users u
            cross join contest_problems p
            left join best b
                on b.user_id = u.user_id
                and b.problem_id = p.id
            group by u.user_id
            order by total_score desc",
    )
    .bind(contest_id)
    .fetch_all(pool)
    .await
    .map_log(TotalVerdict::InvalidRequest)?;

    Ok(leaderboard)
}

/// Gets contest's problem by `contest_id`
/// # Errors
/// Returns an error if `contest_id` is invalid
pub async fn get_contest_public_problems(
    pool: &PgPool,
    contest_id: i64,
) -> Result<Vec<PublicProblemConfig>, TotalVerdict> {
    sqlx::query_as("select * from problems where contest_id = $1")
        .bind(contest_id)
        .fetch_all(pool)
        .await
        .map_log(TotalVerdict::InvalidRequest)
}

/// Gets all contests
/// # Errors
/// Returns an error if `contest_id` is invalid
pub async fn get_contests(pool: &PgPool) -> Result<Vec<DatabaseContestConfig>, TotalVerdict> {
    sqlx::query_as("select * from contests")
        .fetch_all(pool)
        .await
        .map_log(TotalVerdict::InvalidRequest)
}

/// Gets a contest by given id
/// # Errors
/// Returns an error if `contest_id` is invalid
pub async fn get_contest_by_id(
    pool: &PgPool,
    contest_id: i64,
) -> Result<DatabaseContestConfig, TotalVerdict> {
    sqlx::query_as("select * from contests where id = $1")
        .bind(contest_id)
        .fetch_one(pool)
        .await
        .map_log(TotalVerdict::InvalidRequest)
}

/// Get's problem's config from `problems` table by bigen id
/// # Errors
/// Returns an error if `problem_id` is invalid
pub async fn get_problem_by_id(
    pool: &PgPool,
    problem_id: i64,
) -> Result<DatabaseProblemConfig, TotalVerdict> {
    let config = sqlx::query_as(
        "select
                c.id,
                c.owner_id,
                c.contest_id,
                c.problem_index,
                c.name,
                c.time_limit_ms,
                c.memory_limit_mb,
                c.checker_path,
                c.tests_path,
                c.created_at,
                coalesce(
                    json_agg(
                        json_build_object(
                            'type', v.type,
                            'tests', v.tests,
                            'score', v.score,
                            'depends_on', v.depends_on
                        ) order by v.subgroup_index
                    ), '[]'
                ) as subgroups
            from problems c
            left join problems_subgroups v on v.problem_id = c.id
            where c.id = $1
            group by c.id, c.owner_id, c.contest_id, c.problem_index, c.name, c.time_limit_ms, c.memory_limit_mb, c.checker_path, c.tests_path, c.created_at
        ",
    )
    .bind(problem_id)
    .fetch_one(pool)
    .await
    .map_log(TotalVerdict::InvalidRequest)?;

    Ok(config)
}

/// Inserts a submission to `submissions` table and returns it's id
/// # Errors
/// Returns an error if `problem_id` is invalid
pub async fn insert_submission(
    pool: &PgPool,
    user_id: i64,
    problem_id: i64,
) -> Result<i64, TotalVerdict> {
    let submission_id = sqlx::query_scalar(
        "insert into submissions (problem_id, user_id, total_verdict, total_score) 
          values ($1, $2, $3, $4) returning id",
    )
    .bind(problem_id)
    .bind(user_id)
    .bind(TotalVerdict::Pending)
    .bind(0)
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
    sqlx::query(
        "update submissions set total_verdict = $1, total_score = $2 
            where id = $3",
    )
    .bind(verdict)
    .bind(score)
    .bind(submission_id)
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
    sqlx::query(
        "insert into submissions_subgroups_results (subgroup_id, submission_id, subgroup_verdict, test, score, checker_msg)
            values ($1, $2, $3::subgroup_verdict, $4, $5, $6)",
    )
        .bind(subgroup_id)
        .bind(submission_id)
        .bind(SubgroupVerdict::Testing)
        .bind(0)
        .bind(0)
        .bind("")
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
    sqlx::query(
        "update submissions_subgroups_results set subgroup_verdict = $1::subgroup_verdict, test = $2, score = $3, checker_msg = $4 
            where submission_id = $5 and subgroup_id = $6",
    )
        .bind(verdict)
        .bind(test)
        .bind(score)
        .bind(checker_msg)
        .bind(submission_id)
        .bind(subgroup_id)
        .execute(pool)
        .await
        .map_log(TotalVerdict::InvalidRequest)?;
    Ok(())
}

/// Gets all user's submissions
/// # Errors
/// Returns an error if `user_id` is invalid
pub async fn get_all_user_submissions(
    pool: &PgPool,
    user_id: i64,
) -> Result<Vec<Submission>, TotalVerdict> {
    let submissions = sqlx::query_as(
        "select * from submissions
                where user_id = $1",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await
    .map_log(TotalVerdict::InvalidRequest)?;
    Ok(submissions)
}

/// Gets user's submissions for contest
/// # Errors
/// Returns an error if `user_id` or `contest_id` is invalid
pub async fn get_contest_user_submissions(
    pool: &PgPool,
    user_id: i64,
    contest_id: i64,
) -> Result<Vec<Submission>, TotalVerdict> {
    let submissions = sqlx::query_as(
        "select s.* 
                from submissions s
                join problems p on p.id = s.problem_id
                where p.contest_id = $2
                    and s.user_id = $1",
    )
    .bind(user_id)
    .bind(contest_id)
    .fetch_all(pool)
    .await
    .map_log(TotalVerdict::InvalidRequest)?;
    Ok(submissions)
}

/// Gets user's submissions for problem
/// # Errors
/// Returns an error if `user_id` or `problem_id` is invalid
pub async fn get_problem_user_submissions(
    pool: &PgPool,
    user_id: i64,
    problem_id: i64,
) -> Result<Vec<Submission>, TotalVerdict> {
    let submissions =
        sqlx::query_as("select * from submissions where user_id = $1 and problem_id = $2")
            .bind(user_id)
            .bind(problem_id)
            .fetch_all(pool)
            .await
            .map_log(TotalVerdict::InvalidRequest)?;
    Ok(submissions)
}
