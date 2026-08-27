use aj_models::{
    errors::AdaJudgeError,
    testing::{Language, SubgroupResult, Submission, TestResult},
    verdicts::{TestingVerdict, Verdict},
};
use models::testing::DatabaseSubmission;
use sqlx::{PgPool, types::Json};

pub async fn create_submission(
    pool: &PgPool,
    user_id: i64,
    problem_id: i64,
    language: &Language,
) -> Result<i64, AdaJudgeError> {
    let submission_id = sqlx::query_scalar!(
        r#"insert into submissions (problem_id, user_id, language, verdict, score)
          values ($1, $2, $3, $4, $5) returning id"#,
        problem_id,
        user_id,
        language.clone() as Language,
        TestingVerdict::Pending as TestingVerdict,
        0.,
    )
    .fetch_one(pool)
    .await
    .map_err(|_| AdaJudgeError::Internal)?;

    Ok(submission_id)
}

pub async fn update_submission(
    pool: &PgPool,
    submission_id: i64,
    verdict: &TestingVerdict,
    score: f64,
) -> Result<(), AdaJudgeError> {
    sqlx::query!(
        r#"update submissions set verdict = $1, score = $2 where id = $3"#,
        verdict.clone() as TestingVerdict,
        score,
        submission_id
    )
    .execute(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
        _ => AdaJudgeError::Internal,
    })?;

    Ok(())
}

pub async fn create_subgroup_result(
    pool: &PgPool,
    submission_id: i64,
    subgroup_index: i32,
) -> Result<(), AdaJudgeError> {
    sqlx::query!(
        r#"insert into submissions_subgroups_results (subgroup_index, submission_id, verdict, test, score)
            values ($1, $2, $3, $4, $5)"#,
            subgroup_index,
            submission_id,
            Verdict::Testing as Verdict,
            0,
            0.,
        )
        .execute(pool)
        .await
        .map_err(|_| AdaJudgeError::Internal)?;

    Ok(())
}

pub async fn update_subgroup_result(
    pool: &PgPool,
    submission_id: i64,
    subgroup_index: i32,
    subgroup_result: &SubgroupResult,
) -> Result<(), AdaJudgeError> {
    sqlx::query!(
        r#"update submissions_subgroups_results set verdict = $1, test = $2, score = $3
            where submission_id = $4 and subgroup_index = $5"#,
        subgroup_result.verdict.clone() as Verdict,
        subgroup_result.test,
        subgroup_result.score,
        submission_id,
        subgroup_index
    )
    .execute(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
        _ => AdaJudgeError::Internal,
    })?;

    Ok(())
}

pub async fn create_test_result(
    pool: &PgPool,
    submission_id: i64,
    test: i32,
    score: Option<f64>,
) -> Result<(), AdaJudgeError> {
    sqlx::query!(
        r#"insert into submissions_tests_results (test, submission_id, verdict, score)
            values ($1, $2, $3, $4)"#,
        test,
        submission_id,
        Verdict::Testing as Verdict,
        score,
    )
    .execute(pool)
    .await
    .map_err(|_| AdaJudgeError::Internal)?;

    Ok(())
}

pub async fn update_test_testing_result(
    pool: &PgPool,
    submission_id: i64,
    test: i32,
    test_result: &TestResult,
) -> Result<(), AdaJudgeError> {
    sqlx::query!(
        r#"update submissions_tests_results set verdict = $1, score = $2
            where submission_id = $3 and test = $4"#,
        test_result.verdict.clone() as Verdict,
        test_result.score,
        submission_id,
        test
    )
    .execute(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
        _ => AdaJudgeError::Internal,
    })?;

    Ok(())
}

pub async fn get_submission(
    pool: &PgPool,
    submission_id: i64,
) -> Result<Submission, AdaJudgeError> {
    let submission: DatabaseSubmission = sqlx::query_as!(
        DatabaseSubmission,
        r#"select
                c.id,
                c.problem_id as "problem_id!",
                c.user_id as "user_id!",
                users.login as user_login,
                c.language as "language!: Language",
                c.verdict as "verdict!: TestingVerdict",
                c.score,
                c.created_at,
                coalesce(s.subgroups_results, '[]') as "subgroups_results!: Json<Vec<SubgroupResult>>",
                coalesce(t.tests_results, '[]') as "tests_results!: Json<Vec<TestResult>>"
            from submissions c
            left join lateral (
                select json_agg(
                    json_build_object(
                        'verdict', verdict,
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
                        'verdict', verdict,
                        'score', score
                    ) order by test
                ) as tests_results
                from submissions_tests_results
                where submission_id = c.id
            ) t on true
            join users on users.id = c.user_id
            where c.id = $1"#,
        submission_id
    )
    .fetch_one(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
        _ => AdaJudgeError::Internal,
    })?;

    Ok(submission.into())
}

pub async fn get_problem_submissions(
    pool: &PgPool,
    user_id: Option<i64>,
    problem_id: i64,
) -> Result<Vec<Submission>, AdaJudgeError> {
    let submissions = match user_id {
        None => sqlx::query_as!(DatabaseSubmission,
            r#"select
                    c.id,
                    c.problem_id as "problem_id!",
                    c.user_id as "user_id!",
                    users.login as user_login,
                    c.language as "language!: Language",
                    c.verdict as "verdict!: TestingVerdict",
                    c.score,
                    c.created_at,
                    coalesce(s.subgroups_results, '[]') as "subgroups_results!: Json<Vec<SubgroupResult>>",
                    coalesce(t.tests_results, '[]') as "tests_results!: Json<Vec<TestResult>>"
                from submissions c
                left join lateral (
                    select json_agg(
                        json_build_object(
                            'verdict', verdict,
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
                            'verdict', verdict,
                            'score', score
                        ) order by test
                    ) as tests_results
                    from submissions_tests_results
                    where submission_id = c.id
                ) t on true
                join users on users.id = c.user_id
                    join problems p on p.id = c.problem_id
                where c.problem_id = $1 order by c.id desc"#,
                problem_id
        )
        .fetch_all(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
            _ => AdaJudgeError::Internal,
        })?,
    Some(user_id) => sqlx::query_as!(DatabaseSubmission,
        r#"select
                c.id,
                c.problem_id as "problem_id!",
                c.user_id as "user_id!",
                users.login as user_login,
                c.language as "language!: Language",
                c.verdict as "verdict!: TestingVerdict",
                c.score,
                c.created_at,
                coalesce(s.subgroups_results, '[]') as "subgroups_results!: Json<Vec<SubgroupResult>>",
                coalesce(t.tests_results, '[]') as "tests_results!: Json<Vec<TestResult>>"
            from submissions c
            left join lateral (
                select json_agg(
                    json_build_object(
                        'verdict', verdict,
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
                        'verdict', verdict,
                        'score', score
                    ) order by test
                ) as tests_results
                from submissions_tests_results
                where submission_id = c.id
            ) t on true
            join users on users.id = c.user_id
                join problems p on p.id = c.problem_id
            where c.problem_id = $1 and c.user_id = $2 order by c.id desc"#,
            problem_id,
            user_id
        )
        .fetch_all(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
            _ => AdaJudgeError::Internal,
        })?
    }
    .iter()
    .map(|x| x.clone().into())
    .collect();

    Ok(submissions)
}

pub async fn delete_problem_subgroups_results(
    pool: &PgPool,
    problem_id: i64,
) -> Result<(), AdaJudgeError> {
    sqlx::query!(
        r#"delete from submissions_subgroups_results r
            using submissions s
            where r.submission_id = s.id
            and s.problem_id = $1"#,
        problem_id
    )
    .execute(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
        _ => AdaJudgeError::Internal,
    })?;
    Ok(())
}

pub async fn delete_problem_tests_results(
    pool: &PgPool,
    problem_id: i64,
) -> Result<(), AdaJudgeError> {
    sqlx::query!(
        r#"delete from submissions_tests_results r
            using submissions s
            where r.submission_id = s.id
            and s.problem_id = $1"#,
        problem_id,
    )
    .execute(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
        _ => AdaJudgeError::Internal,
    })?;
    Ok(())
}

pub async fn make_submissions_pending(pool: &PgPool, problem_id: i64) -> Result<(), AdaJudgeError> {
    sqlx::query!(
        r#"update submissions set verdict = 'pending'
        where problem_id = $1"#,
        problem_id,
    )
    .execute(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
        _ => AdaJudgeError::Internal,
    })?;
    Ok(())
}
