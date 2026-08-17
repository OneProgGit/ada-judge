use aj_models::{
    errors::AdaJudgeError,
    problems::{ProblemConfig, ProblemQuestion, ProblemQuestionRequest, PublicProblemConfig},
};
use models::problems::DatabaseProblemConfig;
use sqlx::PgPool;

pub async fn get_problem(
    pool: &PgPool,
    problem_id: i64,
) -> Result<DatabaseProblemConfig, AdaJudgeError> {
    let config = sqlx::query_as(
        r#"select
                c.id,
                c.owner_id,
                c.type,
                c.testing_type,
                c.contest_id,
                c.index,
                c.name_ru,
                c.name_en,
                c.time_limit_ms,
                c.memory_limit_mb,
                c.checker_path,
                c.checker_lang,
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
            group by c.id
        "#,
    )
    .bind(problem_id)
    .fetch_one(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
        _ => AdaJudgeError::Internal,
    })?;

    Ok(config)
}

pub async fn get_problems(
    pool: &PgPool,
    user_id: Option<i64>,
) -> Result<Vec<PublicProblemConfig>, AdaJudgeError> {
    let problems = match user_id {
        None => sqlx::query_as::<_, DatabaseProblemConfig>(
            r#"select
                c.id,
                c.owner_id,
                c.type,
                c.testing_type,
                c.contest_id,
                c.index,
                c.name_ru,
                c.name_en,
                c.time_limit_ms,
                c.memory_limit_mb,
                c.checker_path,
                c.checker_lang,
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
            order by id desc"#,
        )
        .fetch_all(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
            _ => AdaJudgeError::Internal,
        })?,
        Some(user_id) => sqlx::query_as::<_, DatabaseProblemConfig>(
            r#"select
                    c.id,
                    c.owner_id,
                    c.type,
                    c.testing_type,
                    c.contest_id,
                    c.index,
                    c.name_ru,
                    c.name_en,
                    c.time_limit_ms,
                    c.memory_limit_mb,
                    c.checker_path,
                    c.checker_lang,
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
                where c.owner_id = $1 order by index"#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
            _ => AdaJudgeError::Internal,
        })?,
    }
    .iter()
    .map(|x| x.clone().into())
    .collect();

    Ok(problems)
}

pub async fn create_problem(
    pool: &PgPool,
    owner_id: i64,
    problem: &ProblemConfig,
) -> Result<i64, AdaJudgeError> {
    let mut tx = pool.begin().await.map_err(|_| AdaJudgeError::Internal)?;

    let problem_id: i64 = sqlx::query_scalar(
        r#"insert into problems (owner_id, type, testing_type, contest_id, index,
                                name_ru, name_en, time_limit_ms, memory_limit_mb,
                                checker_path, checker_lang, tests_path) values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) returning id"#,
    )
    .bind(owner_id)
    .bind(&problem.r#type)
    .bind(&problem.testing_type)
    .bind(problem.contest_id)
    .bind(problem.index)
    .bind(&problem.name_ru)
    .bind(&problem.name_en)
    .bind(problem.time_limit_ms)
    .bind(problem.memory_limit_mb)
    .bind(&problem.checker_path)
    .bind(&problem.checker_lang)
    .bind(&problem.tests_path)
    .fetch_one(&mut *tx)
    .await
    .map_err(|_| AdaJudgeError::Internal)?;

    for (i, subgroup) in problem.subgroups.iter().enumerate() {
        if let Some(score) = subgroup.score {
            sqlx::query(
                r#"insert into problems_subgroups (problem_id, subgroup_index,
                type, tests, score, depends_on) values ($1, $2, $3, $4, $5, $6)"#,
            )
            .bind(problem_id)
            .bind(i as i64)
            .bind(&subgroup.r#type)
            .bind(&subgroup.tests)
            .bind(score)
            .bind(
                subgroup
                    .depends_on
                    .iter()
                    .map(|x| *x as i32)
                    .collect::<Vec<i32>>(),
            )
            .execute(&mut *tx)
            .await
            .map_err(|_| AdaJudgeError::Internal)?;
        } else if let Some(score_per_test) = subgroup.score_per_test {
            sqlx::query(
                r#"insert into problems_subgroups (problem_id, subgroup_index,
                type, tests, score_per_test, depends_on) values ($1, $2, $3, $4, $5, $6)"#,
            )
            .bind(problem_id)
            .bind(i as i64)
            .bind(&subgroup.r#type)
            .bind(&subgroup.tests)
            .bind(score_per_test)
            .bind(
                subgroup
                    .depends_on
                    .iter()
                    .map(|x| *x as i32)
                    .collect::<Vec<i32>>(),
            )
            .execute(&mut *tx)
            .await
            .map_err(|_| AdaJudgeError::Internal)?;
        } else {
            unreachable!()
        }
    }

    tx.commit().await.map_err(|_| AdaJudgeError::Internal)?;

    Ok(problem_id)
}

pub async fn update_problem(
    pool: &PgPool,
    problem_id: i64,
    problem: &ProblemConfig,
) -> Result<(), AdaJudgeError> {
    let mut tx = pool.begin().await.map_err(|_| AdaJudgeError::Internal)?;

    sqlx::query(
        r#"update problems set type = $1, testing_type = $2, contest_id = $3, index = $4,
                                name_ru = $5, name_en = $6, time_limit_ms = $7, memory_limit_mb = $8,
                                checker_path = $9, checker_lang = $10, tests_path = $11 where id = $12"#,
    )
    .bind(&problem.r#type)
    .bind(&problem.testing_type)
    .bind(problem.contest_id)
    .bind(problem.index)
    .bind(&problem.name_ru)
    .bind(&problem.name_en)
    .bind(problem.time_limit_ms)
    .bind(problem.memory_limit_mb)
    .bind(&problem.checker_path)
    .bind(&problem.checker_lang)
    .bind(&problem.tests_path)
    .bind(problem_id)
    .execute(&mut *tx)
    .await
    .map_err(|e| match e {
            sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
            _ => AdaJudgeError::Internal,
        })?;

    sqlx::query(r#"delete from problems_subgroups where problem_id = $1"#)
        .bind(problem_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
            _ => AdaJudgeError::Internal,
        })?;

    for (i, subgroup) in problem.subgroups.iter().enumerate() {
        if let Some(score) = subgroup.score {
            sqlx::query(
                r#"insert into problems_subgroups (problem_id, subgroup_index,
                type, tests, score, depends_on) values ($1, $2, $3, $4, $5, $6)"#,
            )
            .bind(problem_id)
            .bind(i as i64)
            .bind(&subgroup.r#type)
            .bind(&subgroup.tests)
            .bind(score)
            .bind(
                subgroup
                    .depends_on
                    .iter()
                    .map(|x| *x as i32)
                    .collect::<Vec<i32>>(),
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
                _ => AdaJudgeError::Internal,
            })?;
        } else if let Some(score_per_test) = subgroup.score_per_test {
            sqlx::query(
                r#"insert into problems_subgroups (problem_id, subgroup_index,
                type, tests, score_per_test, depends_on) values ($1, $2, $3, $4, $5, $6)"#,
            )
            .bind(problem_id)
            .bind(i as i64)
            .bind(&subgroup.r#type)
            .bind(&subgroup.tests)
            .bind(score_per_test)
            .bind(
                subgroup
                    .depends_on
                    .iter()
                    .map(|x| *x as i32)
                    .collect::<Vec<i32>>(),
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
                _ => AdaJudgeError::Internal,
            })?;
        } else {
            unreachable!()
        }
    }

    tx.commit().await.map_err(|_| AdaJudgeError::Internal)?;

    Ok(())
}

pub async fn delete_problem(pool: &PgPool, problem_id: i64) -> Result<(), AdaJudgeError> {
    sqlx::query(r#"delete from problems where id = $1"#)
        .bind(problem_id)
        .execute(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
            _ => AdaJudgeError::Internal,
        })?;

    Ok(())
}

pub async fn create_problem_question(
    pool: &PgPool,
    user_id: i64,
    problem_id: i64,
    question: &ProblemQuestionRequest,
) -> Result<(), AdaJudgeError> {
    sqlx::query(
        r#"insert into problems_questions (owner_id, problem_id, title, text) values ($1, $2, $3, $4)"#,
    )
    .bind(user_id)
    .bind(problem_id)
    .bind(&question.title)
    .bind(&question.text)
    .fetch_one(pool)
    .await
    .map_err(|_| AdaJudgeError::Internal)?;

    Ok(())
}

pub async fn answer_problem_question(
    pool: &PgPool,
    question_id: i64,
    answer: &str,
) -> Result<(), AdaJudgeError> {
    sqlx::query(r#"update problems_questions set answer = $1 where id = $2"#)
        .bind(answer)
        .bind(question_id)
        .execute(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
            _ => AdaJudgeError::Internal,
        })?;

    Ok(())
}

pub async fn delete_problem_question(pool: &PgPool, question_id: i64) -> Result<(), AdaJudgeError> {
    sqlx::query(r#"delete from problems_questions where id = $1"#)
        .bind(question_id)
        .execute(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
            _ => AdaJudgeError::Internal,
        })?;

    Ok(())
}

pub async fn get_problem_question(
    pool: &PgPool,
    question_id: i64,
) -> Result<ProblemQuestion, AdaJudgeError> {
    sqlx::query_as(r#"select * from problems_questions where id = $1"#)
        .bind(question_id)
        .fetch_one(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
            _ => AdaJudgeError::Internal,
        })
}

pub async fn get_problem_questions(
    pool: &PgPool,
    user_id: Option<i64>,
    problem_id: i64,
) -> Result<Vec<ProblemQuestion>, AdaJudgeError> {
    let questions = match user_id {
        None => sqlx::query_as(
            r#"select * from problems_questions where problem_id = $1 order by id desc"#,
        )
        .bind(problem_id)
        .fetch_all(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
            _ => AdaJudgeError::Internal,
        })?,
        Some(user_id) => sqlx::query_as(
            r#"select * from problems_questions where owner_id = $1 and problem_id = $2 order by id desc"#,
        )
        .bind(user_id)
        .bind(problem_id)
        .fetch_all(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
            _ => AdaJudgeError::Internal,
        })?,
    };

    Ok(questions)
}
