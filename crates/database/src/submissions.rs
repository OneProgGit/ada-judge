//! Database tools for submissions

use ada_judge_public_models::{
    testing::{Language, SubgroupResult},
    verdicts::{SubgroupVerdict, TotalVerdict},
};
use models::testing::DatabaseSubmission;
use sqlx::PgPool;
use tools::map::MapLogExt;

/// Inserts a submission to `submissions` table and returns it's id
/// # Errors
/// Returns an error if `problem_id` is invalid
pub async fn insert_submission(
    pool: &PgPool,
    user_id: i64,
    problem_id: i64,
    language: &Language,
) -> Result<i64, TotalVerdict> {
    let submission_id = sqlx::query_scalar(
        "insert into submissions (problem_id, user_id, language, total_verdict, total_score)
          values ($1, $2, $3, $4, $5) returning id",
    )
    .bind(problem_id)
    .bind(user_id)
    .bind(language)
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
/// Returns an error if `submission_id` is invalid. `TODO`: return an error if `subgroup_index` is out of range
pub async fn insert_subgroup_testing_result(
    pool: &PgPool,
    submission_id: i64,
    subgroup_index: i32,
) -> Result<(), TotalVerdict> {
    sqlx::query(
        "insert into submissions_subgroups_results (subgroup_index, submission_id, subgroup_verdict, test, score)
            values ($1, $2, $3, $4, $5)",
    )
        .bind(subgroup_index)
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
/// Returns an error if `submission_id` is invalid. `TODO`: return an error if `subgroup_index` is out of range
pub async fn update_subgroup_testing_result(
    pool: &PgPool,
    submission_id: i64,
    subgroup_index: i32,
    subgroup_result: &SubgroupResult,
) -> Result<(), TotalVerdict> {
    sqlx::query(
        "update submissions_subgroups_results set subgroup_verdict = $1, test = $2, score = $3
            where submission_id = $4 and subgroup_index = $5",
    )
    .bind(&subgroup_result.subgroup_verdict)
    .bind(subgroup_result.test)
    .bind(subgroup_result.score)
    .bind(submission_id)
    .bind(subgroup_index)
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
) -> Result<Vec<DatabaseSubmission>, TotalVerdict> {
    let submissions = sqlx::query_as(
        "select
                c.id,
                c.problem_id,
                c.user_id,
                c.language,
                c.total_verdict,
                c.total_score,
                c.created_at,
                coalesce(
                    json_agg(
                        json_build_object(
                            'subgroup_verdict', v.subgroup_verdict,
                            'test', v.test,
                            'score', v.score
                        ) order by v.subgroup_index
                    ) filter (where v.submission_id is not null),
                    '[]'
                ) as subgroups_results
            from submissions c
            left join submissions_subgroups_results v on v.submission_id = c.id
            where c.user_id = $1
            group by c.id,
                c.problem_id,
                c.user_id,
                c.language,
                c.total_verdict,
                c.total_score,
                c.created_at",
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
) -> Result<Vec<DatabaseSubmission>, TotalVerdict> {
    let submissions = sqlx::query_as(
        "select
                c.id,
                c.problem_id,
                c.user_id,
                c.language,
                c.total_verdict,
                c.total_score,
                c.created_at,
                coalesce(
                    json_agg(
                        json_build_object(
                            'subgroup_verdict', v.subgroup_verdict,
                            'test', v.test,
                            'score', v.score
                        ) order by v.subgroup_index
                    ) filter (where v.submission_id is not null),
                    '[]'
                ) as subgroups_results
            from submissions c
            left join submissions_subgroups_results v on v.submission_id = c.id
            join problems p on p.id = c.problem_id
            where c.user_id = $1 and p.contest_id = $2
            group by c.id,
                c.problem_id,
                c.user_id,
                c.language,
                c.total_verdict,
                c.total_score,
                c.created_at",
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
) -> Result<Vec<DatabaseSubmission>, TotalVerdict> {
    let submissions = sqlx::query_as(
        "select
                c.id,
                c.problem_id,
                c.user_id,
                c.language,
                c.total_verdict,
                c.total_score,
                c.created_at,
                coalesce(
                    json_agg(
                        json_build_object(
                            'subgroup_verdict', v.subgroup_verdict,
                            'test', v.test,
                            'score', v.score
                        ) order by v.subgroup_index
                    ) filter (where v.submission_id is not null),
                    '[]'
                ) as subgroups_results
            from submissions c
            left join submissions_subgroups_results v on v.submission_id = c.id
            where c.user_id = $1 and c.problem_id = $2
            group by c.id,
                c.problem_id,
                c.user_id,
                c.language,
                c.total_verdict,
                c.total_score,
                c.created_at",
    )
    .bind(user_id)
    .bind(problem_id)
    .fetch_all(pool)
    .await
    .map_log(TotalVerdict::InvalidRequest)?;

    Ok(submissions)
}
