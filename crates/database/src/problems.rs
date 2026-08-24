use aj_models::{
    errors::AdaJudgeError,
    problems::{
        ProblemConfig, ProblemQuestion, ProblemQuestionRequest, ProblemTestingType, ProblemType,
        PublicProblemConfig, Subgroup, SubgroupType,
    },
    testing::Language,
};
use models::problems::DatabaseProblemConfig;
use sqlx::{PgPool, types::Json};

pub async fn get_problem(
    pool: &PgPool,
    problem_id: i64,
) -> Result<DatabaseProblemConfig, AdaJudgeError> {
    let config = sqlx::query_as!(
        DatabaseProblemConfig,
        r#"select
                c.id,
                c.owner_id,
                users.login as owner_login,
                c.type as "type!: ProblemType",
                c.testing_type as "testing_type!: ProblemTestingType",
                c.contest_id as "contest_id!",
                c.index,
                c.name_ru,
                c.name_en,
                c.time_limit_ms,
                c.memory_limit_mb,
                c.checker_path,
                c.checker_lang as "checker_lang!: Language",
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
                ) as "subgroups!: Json<Vec<Subgroup>>"
            from problems c
            left join problems_subgroups v on v.problem_id = c.id
            left join users on users.id = c.owner_id
            where c.id = $1
            group by c.id, owner_login
        "#,
        problem_id
    )
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
        None => sqlx::query_as!(
            DatabaseProblemConfig,
            r#"select
                    c.id,
                    c.owner_id as "owner_id?",
                    users.login as "owner_login?",
                    c.type as "type!: ProblemType",
                    c.testing_type as "testing_type!: ProblemTestingType",
                    c.contest_id as "contest_id!",
                    c.index,
                    c.name_ru,
                    c.name_en,
                    c.time_limit_ms,
                    c.memory_limit_mb,
                    c.checker_path,
                    c.checker_lang as "checker_lang!: Language",
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
                    ) as "subgroups!: Json<Vec<Subgroup>>"
                from problems c
                left join problems_subgroups v on v.problem_id = c.id
                left join users on users.id = c.owner_id
                group by c.id, users.login
                order by c.id desc"#,
        )
        .fetch_all(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
            _ => AdaJudgeError::Internal,
        })?,
        Some(user_id) => sqlx::query_as!(
            DatabaseProblemConfig,
            r#"select
                    c.id,
                    c.owner_id as "owner_id?",
                    users.login as "owner_login?",
                    c.type as "type!: ProblemType",
                    c.testing_type as "testing_type!: ProblemTestingType",
                    c.contest_id as "contest_id!",
                    c.index,
                    c.name_ru,
                    c.name_en,
                    c.time_limit_ms,
                    c.memory_limit_mb,
                    c.checker_path,
                    c.checker_lang as "checker_lang!: Language",
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
                    ) as "subgroups!: Json<Vec<Subgroup>>"
                from problems c
                left join problems_subgroups v on v.problem_id = c.id
                left join users on users.id = c.owner_id
                where c.owner_id = $1
                group by c.id, users.login
                order by c.id desc"#,
            user_id,
        )
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

    let problem_id: i64 = sqlx::query_scalar!(
        r#"insert into problems (owner_id, type, testing_type, contest_id, index,
                                name_ru, name_en, time_limit_ms, memory_limit_mb,
                                checker_path, checker_lang, tests_path) values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) returning id"#,
        owner_id,
        problem.r#type.clone() as ProblemType,
        problem.testing_type.clone() as ProblemTestingType,
        problem.contest_id,
        problem.index,
        &problem.name_ru,
        &problem.name_en,
        problem.time_limit_ms,
        problem.memory_limit_mb,
        &problem.checker_path,
        problem.checker_lang.clone() as Language,
        &problem.tests_path
    )
    .fetch_one(&mut *tx)
    .await
    .map_err(|_| AdaJudgeError::Internal)?;

    for (i, subgroup) in problem.subgroups.iter().enumerate() {
        if let Some(score) = subgroup.score {
            sqlx::query!(
                r#"insert into problems_subgroups (problem_id, subgroup_index,
                type, tests, score, depends_on) values ($1, $2, $3, $4, $5, $6)"#,
                problem_id,
                i as i64,
                subgroup.r#type.clone() as SubgroupType,
                &subgroup.tests,
                score,
                &subgroup
                    .depends_on
                    .iter()
                    .map(|x| *x as i32)
                    .collect::<Vec<i32>>(),
            )
            .execute(&mut *tx)
            .await
            .map_err(|_| AdaJudgeError::Internal)?;
        } else if let Some(score_per_test) = subgroup.score_per_test {
            sqlx::query!(
                r#"insert into problems_subgroups (problem_id, subgroup_index,
                type, tests, score_per_test, depends_on) values ($1, $2, $3, $4, $5, $6)"#,
                problem_id,
                i as i64,
                subgroup.r#type.clone() as SubgroupType,
                &subgroup.tests,
                score_per_test,
                &subgroup
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

    sqlx::query!(
        r#"update problems set type = $1, testing_type = $2, contest_id = $3, index = $4,
                                name_ru = $5, name_en = $6, time_limit_ms = $7, memory_limit_mb = $8,
                                checker_path = $9, checker_lang = $10, tests_path = $11 where id = $12"#, problem.r#type.clone() as ProblemType,
        problem.testing_type.clone() as ProblemTestingType,
        problem.contest_id,
        problem.index,
        &problem.name_ru,
        &problem.name_en,
        problem.time_limit_ms,
        problem.memory_limit_mb,
        &problem.checker_path,
        problem.checker_lang.clone() as Language,
        &problem.tests_path,
        problem_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| match e {
            sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
            _ => AdaJudgeError::Internal,
        })?;

    sqlx::query!(
        r#"delete from problems_subgroups where problem_id = $1"#,
        problem_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
        _ => AdaJudgeError::Internal,
    })?;

    for (i, subgroup) in problem.subgroups.iter().enumerate() {
        if let Some(score) = subgroup.score {
            sqlx::query!(
                r#"insert into problems_subgroups (problem_id, subgroup_index,
                type, tests, score, depends_on) values ($1, $2, $3, $4, $5, $6)"#,
                problem_id,
                i as i64,
                subgroup.r#type.clone() as SubgroupType,
                &subgroup.tests,
                score,
                &subgroup
                    .depends_on
                    .iter()
                    .map(|x| *x as i32)
                    .collect::<Vec<i32>>(),
            )
            .execute(&mut *tx)
            .await
            .map_err(|_| AdaJudgeError::Internal)?;
        } else if let Some(score_per_test) = subgroup.score_per_test {
            sqlx::query!(
                r#"insert into problems_subgroups (problem_id, subgroup_index,
                type, tests, score_per_test, depends_on) values ($1, $2, $3, $4, $5, $6)"#,
                problem_id,
                i as i64,
                subgroup.r#type.clone() as SubgroupType,
                &subgroup.tests,
                score_per_test,
                &subgroup
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

    Ok(())
}

pub async fn delete_problem(pool: &PgPool, problem_id: i64) -> Result<(), AdaJudgeError> {
    sqlx::query!(r#"delete from problems where id = $1"#, problem_id)
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
    sqlx::query!(
        r#"insert into problems_questions (owner_id, problem_id, title, text) values ($1, $2, $3, $4)"#,
        user_id,
        problem_id,
        &question.title,
        &question.text
    )
    .execute(pool)
    .await
    .map_err(|_| AdaJudgeError::Internal)?;

    Ok(())
}

pub async fn answer_problem_question(
    pool: &PgPool,
    question_id: i64,
    answer: &str,
) -> Result<(), AdaJudgeError> {
    sqlx::query!(
        r#"update problems_questions set answer = $1 where id = $2"#,
        answer,
        question_id
    )
    .execute(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
        _ => AdaJudgeError::Internal,
    })?;

    Ok(())
}

pub async fn delete_problem_question(pool: &PgPool, question_id: i64) -> Result<(), AdaJudgeError> {
    sqlx::query!(
        r#"delete from problems_questions where id = $1"#,
        question_id
    )
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
    sqlx::query_as!(
        ProblemQuestion,
        r#"select c.id as "id!",
        c.owner_id as "owner_id!",
        users.login as "owner_login",
        c.problem_id as "problem_id!",
        c.title,
        c.text,
        c.answer,
        c.created_at
        from problems_questions c
        join users on users.id = c.owner_id
        where c.id = $1"#,
        question_id
    )
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
        None => sqlx::query_as!(
            ProblemQuestion,
            r#"select c.id as "id!",
            c.owner_id as "owner_id!",
            users.login as "owner_login",
            c.problem_id as "problem_id!",
            c.title,
            c.text,
            c.answer,
            c.created_at
            from problems_questions c
            join users on users.id = c.owner_id
            where c.problem_id = $1 order by c.id desc"#,
            problem_id
        )
        .fetch_all(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
            _ => AdaJudgeError::Internal,
        })?,
        Some(user_id) => sqlx::query_as!(
            ProblemQuestion,
            r#"select c.id as "id!",
            c.owner_id as "owner_id!",
            users.login as "owner_login",
            c.problem_id as "problem_id!",
            c.title,
            c.text,
            c.answer,
            c.created_at
            from problems_questions c
            join users on users.id = c.owner_id
            where c.owner_id = $1 and c.problem_id = $2 order by c.id desc"#,
            user_id,
            problem_id
        )
        .fetch_all(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
            _ => AdaJudgeError::Internal,
        })?,
    };

    Ok(questions)
}
