//! Database tools for problems

use ada_judge_public_models::{
    problems::{ProblemType, SubgroupType},
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
                c.contest_id,
                c.problem_index,
                c.name,
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

/// Gets all user's problems. If `user_id` is -1, gets all problems.
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
    contest_id: i64,
    problem_index: i64,
    name: &str,
    time_limit_ms: i32,
    memory_limit_mb: i32,
    checker_path: &str,
    tests_path: &str,
) -> Result<i64, TotalVerdict> {
    let problem_id = sqlx::query_scalar(
        "insert into problems (owner_id, type, contest_id, problem_index,
                                name, time_limit_ms, memory_limit_mb,
                                checker_path, tests_path) values ($1, $2, $3, $4, $5, $6, $7, $8, $9) returning id",
    )
    .bind(owner_id)
    .bind(r#type)
    .bind(contest_id)
    .bind(problem_index)
    .bind(name)
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
    } else if let Some(score_per_test) = score {
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
