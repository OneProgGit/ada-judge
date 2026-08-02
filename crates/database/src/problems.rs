//! Database tools for problems

use aj_models::{
    problems::{ProblemQuestion, ProblemType, SubgroupType},
    verdicts::TotalVerdict,
};
use models::problems::DatabaseProblemConfig;
use sqlx::PgPool;
use tools::map::MapLogExt;

/// Gets problem's config from `problems` table by given id
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
                c.type,
                c.merge_subgroups,
                c.contest_id,
                c.problem_index,
                c.name_ru,
                c.name_en,
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
                            'score_per_test', v.score_per_test,
                            'depends_on', v.depends_on
                        ) order by v.subgroup_index
                    ) filter (where v.problem_id is not null),
                    '[]'
                ) as subgroups
            from problems c
            left join problems_subgroups v on v.problem_id = c.id
            where c.id = $1
            group by c.id,
                c.owner_id,
                c.type,
                c.merge_subgroups,
                c.contest_id,
                c.problem_index,
                c.name_ru,
                c.name_en,
                c.time_limit_ms,
                c.memory_limit_mb,
                c.checker_path,
                c.tests_path,
                c.created_at
        ",
    )
    .bind(problem_id)
    .fetch_one(pool)
    .await
    .map_log(TotalVerdict::InvalidRequest)?;

    Ok(config)
}

/// Gets all user's problems. If `user_id` is None, gets all problems.
/// # Errors
/// Returns an error if `user_id` is invalid
pub async fn get_all_user_problems(
    pool: &PgPool,
    user_id: Option<i64>,
) -> Result<Vec<i64>, TotalVerdict> {
    match user_id {
        None => sqlx::query_as::<_, (i64,)>("select id from problems order by id desc")
            .fetch_all(pool)
            .await
            .map(|rows| rows.iter().map(|(id,)| *id).collect())
            .map_log(TotalVerdict::InvalidRequest),
        Some(user_id) => sqlx::query_as::<_, (i64,)>(
            "select id from problems where owner_id = $1 order by id desc",
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map(|rows| rows.iter().map(|(id,)| *id).collect())
        .map_log(TotalVerdict::InvalidRequest),
    }
}

/// Creates a problem in database.
/// # Errors
/// Returns an error if `owner_id` is invalid
pub async fn create_problem(
    pool: &PgPool,
    owner_id: i64,
    r#type: ProblemType,
    merge_subgroups: bool,
    contest_id: i64,
    problem_index: i64,
    name_ru: &str,
    name_en: &str,
    time_limit_ms: i32,
    memory_limit_mb: i32,
    checker_path: &str,
    tests_path: &str,
) -> Result<i64, TotalVerdict> {
    let problem_id = sqlx::query_scalar(
        "insert into problems (owner_id, type, merge_subgroups, contest_id, problem_index,
                                name_ru, name_en, time_limit_ms, memory_limit_mb,
                                checker_path, tests_path) values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) returning id",
    )
    .bind(owner_id)
    .bind(r#type)
    .bind(merge_subgroups)
    .bind(contest_id)
    .bind(problem_index)
    .bind(name_ru)
    .bind(name_en)
    .bind(time_limit_ms)
    .bind(memory_limit_mb)
    .bind(checker_path)
    .bind(tests_path)
    .fetch_one(pool)
    .await
    .map_log(TotalVerdict::InvalidRequest)?;

    Ok(problem_id)
}

/// Inserts a problem's subgroup into database.
/// # Errors
/// Returns an error if `problem_id` is invalid
pub async fn insert_problem_subgroup(
    pool: &PgPool,
    problem_id: i64,
    subgroup_index: usize,
    r#type: &SubgroupType,
    tests: &Vec<i32>,
    score: Option<i32>,
    score_per_test: Option<i32>,
    depends_on: &Vec<usize>,
) -> Result<(), TotalVerdict> {
    if score.is_some() == score_per_test.is_some() {
        return Err(TotalVerdict::InvalidProblem);
    } else if let Some(score) = score {
        sqlx::query(
            "insert into problems_subgroups (problem_id, subgroup_index,
                                            type, tests, score, depends_on) values ($1, $2, $3, $4, $5, $6)",
        )
        .bind(problem_id)
        .bind(subgroup_index as i64)
        .bind(r#type)
        .bind(tests)
        .bind(score)
        .bind(depends_on.iter().map(|x| *x as i32).collect::<Vec<i32>>())
        .execute(pool)
        .await
        .map_log(TotalVerdict::InvalidRequest)?;

        Ok(())
    } else if let Some(score_per_test) = score_per_test {
        sqlx::query(
            "insert into problems_subgroups (problem_id, subgroup_index,
                                            type, tests, score_per_test, depends_on) values ($1, $2, $3, $4, $5, $6)",
        )
        .bind(problem_id)
        .bind(subgroup_index as i64)
        .bind(r#type)
        .bind(tests)
        .bind(score_per_test)
        .bind(depends_on.iter().map(|x| *x as i32).collect::<Vec<i32>>())
        .execute(pool)
        .await
        .map_log(TotalVerdict::InvalidRequest)?;

        Ok(())
    } else {
        unreachable!()
    }
}

/// Deletes a problem by given id
/// # Errors
/// Returns an error if the problem with this id does not exist
pub async fn delete_problem(pool: &PgPool, problem_id: i64) -> Result<(), TotalVerdict> {
    sqlx::query("delete from problems where id = $1")
        .bind(problem_id)
        .execute(pool)
        .await
        .map_log(TotalVerdict::InvalidRequest)?;

    Ok(())
}

/// Creates a question for a problem by given question data
/// # Errors
/// Returns an error if `owner_id` is invalid
pub async fn create_problem_question(
    pool: &PgPool,
    owner_id: i64,
    problem_id: i64,
    title: &str,
    text: &str,
) -> Result<i64, TotalVerdict> {
    let post_id = sqlx::query_scalar(
        "insert into contests_posts (owner_id, problem_id, title, text) values ($1, $2, $3, $4) returning id",
    )
    .bind(owner_id)
    .bind(problem_id)
    .bind(title)
    .bind(text)
    .fetch_one(pool)
    .await
    .map_log(TotalVerdict::InvalidRequest)?;

    Ok(post_id)
}

/// Updates a question's answer
/// # Errors
/// Returns an error if `question_id` is invalid
pub async fn update_problem_question_answer(
    pool: &PgPool,
    question_id: i64,
    answer: &str,
) -> Result<(), TotalVerdict> {
    sqlx::query("update problems_questions set answer = $1 where id = $2")
        .bind(answer)
        .bind(question_id)
        .execute(pool)
        .await
        .map_log(TotalVerdict::InvalidRequest)?;

    Ok(())
}

/// Deletes a problem's question
/// # Errors
/// Returns an error if `question_id` is invalid
pub async fn delete_problem_question(pool: &PgPool, question_id: i64) -> Result<(), TotalVerdict> {
    sqlx::query("delete from problems_questions where id = $1")
        .bind(question_id)
        .execute(pool)
        .await
        .map_log(TotalVerdict::InvalidRequest)?;

    Ok(())
}

/// Gets a problem's question by given id
/// # Errors
/// Returns an error if `question_id` is invalid
pub async fn get_problem_question_by_id(
    pool: &PgPool,
    question_id: i64,
) -> Result<ProblemQuestion, TotalVerdict> {
    sqlx::query_as("select * from problems_questions where id = $1")
        .bind(question_id)
        .fetch_one(pool)
        .await
        .map_log(TotalVerdict::InvalidRequest)
}

/// Gets all user's problem's questions. If `user_id` is None, gets all problem's questions.
/// # Errors
/// Returns an error if `user_id` or `problem_id` is invalid
pub async fn get_all_user_problem_questions(
    pool: &PgPool,
    user_id: Option<i64>,
    problem_id: i64,
) -> Result<Vec<i64>, TotalVerdict> {
    match user_id {
        None => sqlx::query_as::<_, (i64,)>(
            "select id from problems_questions where problem_id = $1 order by id desc",
        )
        .bind(problem_id)
        .fetch_all(pool)
        .await
        .map(|rows| rows.iter().map(|(id,)| *id).collect())
        .map_log(TotalVerdict::InvalidRequest),
        Some(user_id) => sqlx::query_as::<_, (i64,)>(
            "select id from problems_questions where owner_id = $1 and problem_id = $2 order by id desc",
        )
        .bind(user_id)
        .bind(problem_id)
        .fetch_all(pool)
        .await
        .map(|rows| rows.iter().map(|(id,)| *id).collect())
        .map_log(TotalVerdict::InvalidRequest),
    }
}
