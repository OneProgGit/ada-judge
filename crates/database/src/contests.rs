use aj_models::{
    contests::{
        ContestPost, ContestPostRequest, ContestRequest, LeaderboardRow, PublicContestConfig,
    },
    errors::AdaJudgeError,
    problems::PublicProblemConfig,
};
use models::problems::DatabaseProblemConfig;
use sqlx::PgPool;

pub enum GetContestsMode {
    User(i64),
    NotHidden(i64),
    All,
}

pub async fn get_leaderboard(
    pool: &PgPool,
    contest_id: i64,
) -> Result<Vec<LeaderboardRow>, AdaJudgeError> {
    let leaderboard = sqlx::query_as(
        r#"with default_ranked as (
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
                join contests c on c.id = p.contest_id
                where p.contest_id = $1 and not p.merge_subgroups
                    and s.created_at between c.starts_at and c.ends_at
            ),
            default_best as (
                select user_id, problem_id, total_score
                from default_ranked
                where rn = 1
            ),
            merge_subgroups_best_raw as (
                select
                    s.user_id,
                    s.problem_id,
                    ssr.subgroup_index,
                    max(ssr.score) as best_score
                from submissions s
                join submissions_subgroups_results ssr on ssr.submission_id = s.id
                join problems p on p.id = s.problem_id
                join contests c on c.id = p.contest_id
                where p.contest_id = $1
                    and p.merge_subgroups
                    and s.created_at between c.starts_at and c.ends_at
                group by s.user_id, s.problem_id, ssr.subgroup_index
            ),
            merge_subgroups_best as (
                select
                    user_id,
                    problem_id,
                    sum(best_score)::int as total_score
                from merge_subgroups_best_raw
                group by user_id, problem_id
            ),
            best as (
                select * from default_best
                union all
                select * from merge_subgroups_best
            ),
            users as (
                select distinct s.user_id, u.login
                from submissions s
                join users u on u.id = s.user_id
                join problems p on p.id = s.problem_id
                join contests c on c.id = p.contest_id
                where p.contest_id = $1
                    and s.created_at between c.starts_at and c.ends_at
            ),
            contest_problems as (
                select id, problem_index
                from problems
                where contest_id = $1
            )
            select
                u.user_id,
                u.login,
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
            order by total_score desc"#,
    )
    .bind(contest_id)
    .fetch_all(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
        _ => AdaJudgeError::Internal,
    })?;

    Ok(leaderboard)
}

pub async fn get_problems(
    pool: &PgPool,
    contest_id: i64,
) -> Result<Vec<PublicProblemConfig>, AdaJudgeError> {
    let problems = sqlx::query_as::<_, DatabaseProblemConfig>(
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
            where c.contest_id = $1 order by index"#,
    )
    .bind(contest_id)
    .fetch_all(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
        _ => AdaJudgeError::Internal,
    })?
    .iter()
    .map(|x| x.clone().into())
    .collect();

    Ok(problems)
}

pub async fn get_contests(
    pool: &PgPool,
    mode: GetContestsMode,
) -> Result<Vec<PublicContestConfig>, AdaJudgeError> {
    let contests = match mode {
        GetContestsMode::All => sqlx::query_as(
            r#"select
                    c.id,
                    c.owner_id,
                    c.name_ru,
                    c.name_en,
                    c.statements_url_ru,
                    c.editorial_url_ru,
                    c.statements_url_en,
                    c.editorial_url_en,
                    c.starts_at,
                    c.finishes_at,
                    c.hidden,
                    c.upsolving_enabled,
                    c.solutions_hidden,
                    c.leaderboard_hidden,
                    coalesce(
                        array_agg(co.user_id) filter (where co.user_id is not null),
                        '{}'
                    ) as co_authors from contests c
                    left join contests_co_authors co on co.contest_id = c.id
                    group by c.id
                    order by c.id desc"#,
        )
        .fetch_all(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
            _ => AdaJudgeError::Internal,
        })?,

        GetContestsMode::NotHidden(user_id) => sqlx::query_as(
            r#"select
                    c.id,
                    c.owner_id,
                    c.name_ru,
                    c.name_en,
                    c.statements_url_ru,
                    c.editorial_url_ru,
                    c.statements_url_en,
                    c.editorial_url_en,
                    c.starts_at,
                    c.finishes_at,
                    c.hidden,
                    c.upsolving_enabled,
                    c.solutions_hidden,
                    c.leaderboard_hidden,
                    coalesce(
                        array_agg(co.user_id) filter (where co.user_id is not null),
                        '{}'
                    ) as co_authors from contests c
                    left join contests_co_authors co on co.contest_id = c.id
                    where not c.hidden or c.owner_id = $1
                    or exists(
                        select 1 from contests_co_authors
                        where contest_id = c.id
                            and user_id = $1
                    )
                    group by c.id
                    order by c.id desc"#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
            _ => AdaJudgeError::Internal,
        })?,

        GetContestsMode::User(user_id) => sqlx::query_as(
            r#"select
                    c.id,
                    c.owner_id,
                    c.name_ru,
                    c.name_en,
                    c.statements_url_ru,
                    c.editorial_url_ru,
                    c.statements_url_en,
                    c.editorial_url_en,
                    c.starts_at,
                    c.finishes_at,
                    c.hidden,
                    c.upsolving_enabled,
                    c.solutions_hidden,
                    c.leaderboard_hidden,
                    coalesce(
                        array_agg(co.user_id) filter (where co.user_id is not null),
                        '{}'
                    ) as co_authors from contests c
                    left join contests_co_authors co on co.contest_id = c.id
                    where c.owner_id = $1
                    group by c.id
                    order by c.id desc"#,
        )
        .bind(user_id)
        .fetch_all(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
            _ => AdaJudgeError::Internal,
        })?,
    };

    Ok(contests)
}

pub async fn get_contest(
    pool: &PgPool,
    contest_id: i64,
) -> Result<PublicContestConfig, AdaJudgeError> {
    sqlx::query_as(
        r#"select
                c.id,
                c.owner_id,
                c.name_ru,
                c.name_en,
                c.statements_url_ru,
                c.editorial_url_ru,
                c.statements_url_en,
                c.editorial_url_en,
                c.starts_at,
                c.finishes_at,
                c.hidden,
                c.upsolving_enabled,
                c.solutions_hidden,
                c.leaderboard_hidden,
                coalesce(
                    array_agg(co.user_id) filter (where co.user_id is not null),
                    '{}'
                ) as co_authors from contests c
                left join contests_co_authors co on co.contest_id = c.id
                where c.id = $1
                group by c.id"#,
    )
    .bind(contest_id)
    .fetch_one(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
        _ => AdaJudgeError::Internal,
    })
}

pub async fn create_contest(
    pool: &PgPool,
    user_id: i64,
    contest: &ContestRequest,
) -> Result<(), AdaJudgeError> {
    let mut tx = pool.begin().await.map_err(|_| AdaJudgeError::Internal)?;

    let contest_id: i64 = sqlx::query_scalar(
        r#"insert into contests
            (owner_id, name_ru, name_en, starts_at,
            finishes_at, statements_url_ru, editorial_url_ru, statements_url_en, editorial_url_en, hidden,
            upsolving_enabled, solutions_hidden, leaderboard_hidden) values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) returning id"#,
        )
        .bind(user_id)
        .bind(&contest.name_ru)
        .bind(&contest.name_en)
        .bind(contest.starts_at)
        .bind(contest.finishes_at)
        .bind(&contest.statements_url_ru)
        .bind(&contest.editorial_url_ru)
        .bind(&contest.statements_url_en)
        .bind(&contest.editorial_url_en)
        .bind(contest.hidden)
        .bind(contest.upsolving_enabled)
        .bind(contest.solutions_hidden)
        .bind(contest.leaderboard_hidden)
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| AdaJudgeError::Internal)?;

    for user_id in &contest.co_authors {
        sqlx::query(r#"insert into contests_co_authors (contest_id, user_id) values ($1, $2)"#)
            .bind(contest_id)
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(|_| AdaJudgeError::Internal)?;
    }

    tx.commit().await.map_err(|_| AdaJudgeError::Internal)?;

    Ok(())
}

pub async fn update_contest(
    pool: &PgPool,
    contest_id: i64,
    contest: &ContestRequest,
) -> Result<(), AdaJudgeError> {
    let mut tx = pool.begin().await.map_err(|_| AdaJudgeError::Internal)?;

    sqlx::query(r#"update contests set name_ru = $1, name_en = $2, starts_at = $3,
                    finishes_at = $4, statements_url_ru = $5, editorial_url_ru = $6, statements_url_en = $7,
                    editorial_url_en = $8, hidden = $9, upsolving_enabled = $10,
                    solutions_hidden = $11, leaderboard_hidden = $12 where id = $13"#)
            .bind(&contest.name_ru)
            .bind(&contest.name_en)
            .bind(contest.starts_at)
            .bind(contest.finishes_at)
            .bind(&contest.statements_url_ru)
            .bind(&contest.editorial_url_ru)
            .bind(&contest.statements_url_en)
            .bind(&contest.editorial_url_en)
            .bind(contest.hidden)
            .bind(contest.upsolving_enabled)
            .bind(contest.solutions_hidden)
            .bind(contest.leaderboard_hidden)
            .bind(contest_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| match e {
            sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
            _ => AdaJudgeError::Internal,
        })?;

    sqlx::query(r#"delete from contests_co_authors where contest_id = $1"#)
        .bind(contest_id)
        .execute(&mut *tx)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
            _ => AdaJudgeError::Internal,
        })?;

    for user_id in &contest.co_authors {
        sqlx::query(r#"insert into contests_co_authors (contest_id, user_id) values ($1, $2)"#)
            .bind(contest_id)
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
                _ => AdaJudgeError::Internal,
            })?;
    }

    tx.commit().await.map_err(|_| AdaJudgeError::Internal)?;

    Ok(())
}

pub async fn delete_contest(pool: &PgPool, contest_id: i64) -> Result<(), AdaJudgeError> {
    sqlx::query(r#"delete from contests where id = $1"#)
        .bind(contest_id)
        .execute(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
            _ => AdaJudgeError::Internal,
        })?;

    Ok(())
}

pub async fn create_contest_post(
    pool: &PgPool,
    user_id: i64,
    contest_id: i64,
    post: &ContestPostRequest,
) -> Result<(), AdaJudgeError> {
    sqlx::query(
        r#"insert into contests_posts (owner_id, contest_id, title_ru,
            text_ru, title_en, text_en) values ($1, $2, $3, $4, $5, $6)"#,
    )
    .bind(user_id)
    .bind(contest_id)
    .bind(&post.title_ru)
    .bind(&post.text_ru)
    .bind(&post.title_en)
    .bind(&post.text_en)
    .fetch_one(pool)
    .await
    .map_err(|_| AdaJudgeError::Internal)?;

    Ok(())
}

pub async fn update_contest_post(
    pool: &PgPool,
    post_id: i64,
    post: &ContestPostRequest,
) -> Result<(), AdaJudgeError> {
    sqlx::query(
        r#"update contests_posts set title_ru = $1, text_ru = $2,
                    title_en = $3, text_en = $4 where id = $5"#,
    )
    .bind(&post.title_ru)
    .bind(&post.text_ru)
    .bind(&post.title_en)
    .bind(&post.text_en)
    .bind(post_id)
    .execute(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
        _ => AdaJudgeError::Internal,
    })?;

    Ok(())
}

pub async fn delete_contest_post(pool: &PgPool, post_id: i64) -> Result<(), AdaJudgeError> {
    sqlx::query(r#"delete from contests_posts where id = $1"#)
        .bind(post_id)
        .execute(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
            _ => AdaJudgeError::Internal,
        })?;

    Ok(())
}

pub async fn get_contest_post(pool: &PgPool, post_id: i64) -> Result<ContestPost, AdaJudgeError> {
    sqlx::query_as(r#"select * from contests_posts where id = $1"#)
        .bind(post_id)
        .fetch_one(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
            _ => AdaJudgeError::Internal,
        })
}

pub async fn get_contest_posts(
    pool: &PgPool,
    contest_id: i64,
) -> Result<Vec<ContestPost>, AdaJudgeError> {
    let posts =
        sqlx::query_as(r#"select * from contests_posts where contest_id = $1 order by id desc"#)
            .bind(contest_id)
            .fetch_all(pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
                _ => AdaJudgeError::Internal,
            })?;

    Ok(posts)
}
