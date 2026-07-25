//! Database tools for submissions

use aj_models::{
    testing::{Language, SubgroupResult, TestResult},
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

/// Inserts a subgroup's testing result into `submissions_subgroups_results` table
/// # Errors
/// Returns an error if `submission_id` is invalid.
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
        .execute(pool)
        .await
        .map_log(TotalVerdict::InvalidRequest)?;

    Ok(())
}

/// Inserts a test's testing result into `submissions_tests_results` table
/// # Errors
/// Returns an error if `submission_id` is invalid.
pub async fn insert_test_testing_result(
    pool: &PgPool,
    submission_id: i64,
    test: i32,
    score: Option<i32>,
) -> Result<(), TotalVerdict> {
    sqlx::query(
        "insert into submissions_tests_results (test, submission_id, test_verdict, score)
            values ($1, $2, $3, $4)",
    )
    .bind(test)
    .bind(submission_id)
    .bind(SubgroupVerdict::Testing)
    .bind(score)
    .execute(pool)
    .await
    .map_log(TotalVerdict::InvalidRequest)?;

    Ok(())
}

/// Updates testing result for a subgroup of the problem
/// # Errors
/// Returns an error if `submission_id` is invalid.
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

/// Updates testing result for a test of the problem
/// # Errors
/// Returns an error if `submission_id` is invalid.
pub async fn update_test_testing_result(
    pool: &PgPool,
    submission_id: i64,
    test: i32,
    test_result: &TestResult,
) -> Result<(), TotalVerdict> {
    sqlx::query(
        "update submissions_tests_results set test_verdict = $1, score = $2
            where submission_id = $3 and test = $4",
    )
    .bind(&test_result.test_verdict)
    .bind(test_result.score)
    .bind(submission_id)
    .bind(test)
    .execute(pool)
    .await
    .map_log(TotalVerdict::InvalidRequest)?;

    Ok(())
}

/// Gets user's submission by it's id.
/// # Errors
/// Returns an error if `submission_id` is invalid
pub async fn get_submission(
    pool: &PgPool,
    submission_id: i64,
) -> Result<DatabaseSubmission, TotalVerdict> {
    let submission = sqlx::query_as(
        "select
                c.id,
                c.problem_id,
                c.user_id,
                c.language,
                c.total_verdict,
                c.total_score,
                c.created_at,
                coalesce(s.subgroups_results, '[]') as subgroups_results,
                coalesce(t.tests_results, '[]') as tests_results
            from submissions c
            left join lateral (
                select json_agg(
                    json_build_object(
                        'subgroup_verdict', subgroup_verdict,
                        'test', test,
                        'score', score
                    ) order by subgroup_index
                ) as subgroups_results
                from submissions_subgroups_results
                where submission_id = c.id
            ) s on true
            left join lateral (
                select json_agg(
                    json_build_object(
                        'test_verdict', test_verdict,
                        'score', score
                    ) order by test
                ) as tests_results
                from submissions_tests_results
                where submission_id = c.id
            ) t on true
            where c.id = $1",
    )
    .bind(submission_id)
    .fetch_one(pool)
    .await
    .map_log(TotalVerdict::InvalidRequest)?;

    Ok(submission)
}

/// Gets all user's submissions. If `user_id` is None, gets all submissions.
/// # Errors
/// Returns an error if `user_id` is invalid
pub async fn get_all_user_submissions(
    pool: &PgPool,
    user_id: Option<i64>,
) -> Result<Vec<i64>, TotalVerdict> {
    match user_id {
        None => sqlx::query_as::<_, (i64,)>("select id from submissions order by id desc")
            .fetch_all(pool)
            .await
            .map(|rows| rows.iter().map(|(id,)| *id).collect())
            .map_log(TotalVerdict::InvalidRequest),
        Some(user_id) => sqlx::query_as::<_, (i64,)>(
            "select id from submissions where user_id = $1 order by id desc",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map(|rows| rows.iter().map(|(id,)| *id).collect())
        .map_log(TotalVerdict::InvalidRequest),
    }
}

/// Gets user's submissions for contest. If `user_id` is None, gets all submissions for contest.
/// # Errors
/// Returns an error if `user_id` or `contest_id` is invalid
pub async fn get_user_contest_submissions(
    pool: &PgPool,
    user_id: Option<i64>,
    contest_id: i64,
) -> Result<Vec<i64>, TotalVerdict> {
    match user_id {
        None => sqlx::query_as::<_, (i64,)>(
            "select c.id from submissions c
                join problems p on p.id = c.problem_id
            where p.contest_id = $1 order by c.id desc",
        )
        .bind(contest_id)
        .fetch_all(pool)
        .await
        .map(|rows| rows.iter().map(|(id,)| *id).collect())
        .map_log(TotalVerdict::InvalidRequest),
        Some(user_id) => sqlx::query_as::<_, (i64,)>(
            "select c.id from submissions c
                join problems p on p.id = c.problem_id
            where c.user_id = $1 and p.contest_id = $2 order by c.id desc",
        )
        .bind(user_id)
        .bind(contest_id)
        .fetch_all(pool)
        .await
        .map(|rows| rows.iter().map(|(id,)| *id).collect())
        .map_log(TotalVerdict::InvalidRequest),
    }
}

/// Gets user's submissions for problem. If `user_id` is None, gets all submissions for problem.
/// # Errors
/// Returns an error if `user_id` or `problem_id` is invalid
pub async fn get_user_problem_submissions(
    pool: &PgPool,
    user_id: Option<i64>,
    problem_id: i64,
) -> Result<Vec<i64>, TotalVerdict> {
    match user_id {
        None => sqlx::query_as::<_, (i64,)>(
            "select id from submissions where problem_id = $1 order by id desc",
        )
        .bind(problem_id)
        .fetch_all(pool)
        .await
        .map(|rows| rows.iter().map(|(id,)| *id).collect())
        .map_log(TotalVerdict::InvalidRequest),
        Some(user_id) => sqlx::query_as::<_, (i64,)>(
            "select id from submissions where user_id = $1 and problem_id = $2 order by id desc",
        )
        .bind(user_id)
        .bind(problem_id)
        .fetch_all(pool)
        .await
        .map(|rows| rows.iter().map(|(id,)| *id).collect())
        .map_log(TotalVerdict::InvalidRequest),
    }
}

/// Deletes subgroups' submission results for all submissions for a problem
/// # Errors
/// Returns an error if `problem_id` is invalid
pub async fn delete_subgroups_results_for_problem(
    pool: &PgPool,
    problem_id: i64,
) -> Result<(), TotalVerdict> {
    sqlx::query(
        "delete from submissions_subgroups_results r
            using submissions s
            where r.submission_id = s.id
                and s.problem_id = $1",
    )
    .bind(problem_id)
    .execute(pool)
    .await
    .map_log(TotalVerdict::InvalidRequest)?;
    Ok(())
}

/// Deletes tests' submission results for all submissions for a problem
/// # Errors
/// Returns an error if `problem_id` is invalid
pub async fn delete_tests_results_for_problem(
    pool: &PgPool,
    problem_id: i64,
) -> Result<(), TotalVerdict> {
    sqlx::query(
        "delete from submissions_tests_results r
            using submissions s
            where r.submission_id = s.id
                and s.problem_id = $1",
    )
    .bind(problem_id)
    .execute(pool)
    .await
    .map_log(TotalVerdict::InvalidRequest)?;
    Ok(())
}

/// Sets `Pending` verdict for all submissions for a problem
/// # Errors
/// Returns an error if `problem_id` is invalid
pub async fn set_all_submissions_pending_for_problem(
    pool: &PgPool,
    problem_id: i64,
) -> Result<(), TotalVerdict> {
    sqlx::query(
        "update submissions set total_verdict = 'pending'
            where problem_id = $1",
    )
    .bind(problem_id)
    .execute(pool)
    .await
    .map_log(TotalVerdict::InvalidRequest)?;
    Ok(())
}
