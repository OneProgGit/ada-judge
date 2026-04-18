//! Database tools for contests

use ada_judge_public_models::{contests::LeaderboardRow, verdicts::TotalVerdict};
use models::contests::DatabaseContestConfig;
use sqlx::{
    PgPool,
    types::chrono::{DateTime, Utc},
};
use tools::map::MapLogExt;

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

/// Gets contest's problems by `contest_id`
/// # Errors
/// Returns an error if `contest_id` is invalid
pub async fn get_contest_problems(
    pool: &PgPool,
    contest_id: i64,
) -> Result<Vec<i64>, TotalVerdict> {
    sqlx::query_as::<_, (i64,)>(
        "select id from problems where contest_id = $1 order by problem_index",
    )
    .bind(contest_id)
    .fetch_all(pool)
    .await
    .map(|rows| rows.iter().map(|(id,)| *id).collect())
    .map_log(TotalVerdict::InvalidRequest)
}

/// Gets all contests starting with new ones
/// # Errors
/// Returns an error if `contest_id` is invalid
pub async fn get_contests(pool: &PgPool) -> Result<Vec<i64>, TotalVerdict> {
    sqlx::query_as::<_, (i64,)>("select id from contests order by id desc")
        .fetch_all(pool)
        .await
        .map(|rows| rows.iter().map(|(id,)| *id).collect())
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

/// Creates a contest by given contest data
/// # Errors
/// Returns an error if `owner_id` is invalid
pub async fn create_contest(
    pool: &PgPool,
    owner_id: i64,
    name: &str,
    starts_at: &DateTime<Utc>,
    ends_at: &DateTime<Utc>,
) -> Result<i64, TotalVerdict> {
    let contest_id = sqlx::query_scalar(
        "insert into contests (owner_id, name, starts_at, ends_at) values ($1, $2, $3, $4) returning id",
    )
    .bind(owner_id)
    .bind(name)
    .bind(starts_at)
    .bind(ends_at)
    .fetch_one(pool)
    .await
    .map_log(TotalVerdict::InvalidRequest)?;

    Ok(contest_id)
}

/// Updates a contest by given contest id and contest data
/// # Errors
/// Returns an error if `contest_id` is invalid
pub async fn update_contest(
    pool: &PgPool,
    contest_id: i64,
    name: &str,
    starts_at: &DateTime<Utc>,
    ends_at: &DateTime<Utc>,
) -> Result<(), TotalVerdict> {
    sqlx::query("update contests set name = $1, starts_at = $2, ends_at = $3 where id = $4")
        .bind(name)
        .bind(starts_at)
        .bind(ends_at)
        .bind(contest_id)
        .execute(pool)
        .await
        .map_log(TotalVerdict::InvalidRequest)?;

    Ok(())
}
