use aj_models::{
    contests::{
        ContestPost, ContestPostRequest, ContestRequest, LeaderboardRow, PublicContestConfig,
    },
    errors::AdaJudgeError,
    problems::{ProblemTestingType, ProblemType, PublicProblemConfig, Subgroup},
    testing::Language,
};
use models::problems::DatabaseProblemConfig;
use sqlx::{PgPool, types::Json};

pub enum GetContestsMode {
    User(i64),
    NotHidden(i64),
    All,
}

pub async fn get_leaderboard(
    pool: &PgPool,
    contest_id: i64,
) -> Result<Vec<LeaderboardRow>, AdaJudgeError> {
    let leaderboard = sqlx::query_as!(
        LeaderboardRow,
        r#"with default_ranked as (
                select
                    s.user_id,
                    s.problem_id,
                    s.score,
                    row_number() over (
                        partition by s.user_id, s.problem_id
                        order by s.score desc
                    ) as rn
                from submissions s
                join problems p on p.id = s.problem_id
                join contests c on c.id = p.contest_id
                where p.contest_id = $1 and p.testing_type = 'ioi'
                    and s.created_at between c.starts_at and c.finishes_at
            ),
            default_best as (
                select user_id, problem_id, score
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
                    and p.testing_type = 'ioi_merge_subgroups'
                    and s.created_at between c.starts_at and c.finishes_at
                group by s.user_id, s.problem_id, ssr.subgroup_index
            ),
            merge_subgroups_best as (
                select
                    user_id,
                    problem_id,
                    sum(best_score)::int as score
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
                    and s.created_at between c.starts_at and c.finishes_at
            ),
            contest_problems as (
                select id, index
                from problems
                where contest_id = $1
            )
            select
                u.user_id as "user_id!",
                u.login as user_login,
                array_agg(
                    coalesce(b.score, 0::double precision)
                    order by p.index
                )
                as "scores!",
                sum(coalesce(b.score, 0)) as "total_score!"
            from users u
            cross join contest_problems p
            left join best b
                on b.user_id = u.user_id
                and b.problem_id = p.id
            group by u.user_id, user_login
            order by 4 desc"#,
        contest_id
    )
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
    let problems = sqlx::query_as!(
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
            where c.contest_id = $1
            group by c.id, owner_login
            order by index"#,
        contest_id
    )
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
        GetContestsMode::All => sqlx::query_as!(
            PublicContestConfig,
            r#"select
                    c.id,
                    c.owner_id,
                    users.login as owner_login,
                    c.name_ru as "name_ru!",
                    c.name_en as "name_en!",
                    c.statements_url_ru as "statements_url_ru!",
                    c.editorial_url_ru as "editorial_url_ru!",
                    c.statements_url_en as "statements_url_en!",
                    c.editorial_url_en as "editorial_url_en!",
                    c.starts_at,
                    c.finishes_at,
                    c.hidden,
                    c.upsolving_enabled,
                    c.solutions_hidden,
                    c.leaderboard_hidden,
                    c.created_at,
                    coalesce(
                        array_agg(co.user_id) filter (where co.user_id is not null),
                        '{}'
                    ) as "co_authors!" from contests c
                    left join contests_co_authors co on co.contest_id = c.id
                    left join users on users.id = c.owner_id
                    group by c.id, owner_login
                    order by c.id desc"#,
        )
        .fetch_all(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
            _ => AdaJudgeError::Internal,
        })?,

        GetContestsMode::NotHidden(user_id) => sqlx::query_as!(
            PublicContestConfig,
            r#"select
                    c.id,
                    c.owner_id,
                    users.login as owner_login,
                    c.name_ru as "name_ru!",
                    c.name_en as "name_en!",
                    c.statements_url_ru as "statements_url_ru!",
                    c.editorial_url_ru as "editorial_url_ru!",
                    c.statements_url_en as "statements_url_en!",
                    c.editorial_url_en as "editorial_url_en!",
                    c.starts_at,
                    c.finishes_at,
                    c.hidden,
                    c.upsolving_enabled,
                    c.solutions_hidden,
                    c.leaderboard_hidden,
                    c.created_at,
                    coalesce(
                        array_agg(co.user_id) filter (where co.user_id is not null),
                        '{}'
                    ) as "co_authors!" from contests c
                    left join contests_co_authors co on co.contest_id = c.id
                    left join users on users.id = c.owner_id
                    where not c.hidden or c.owner_id = $1
                    or exists(
                        select 1 from contests_co_authors
                        where contest_id = c.id
                            and user_id = $1
                    )
                    group by c.id, owner_login
                    order by c.id desc"#,
            user_id
        )
        .fetch_all(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
            _ => AdaJudgeError::Internal,
        })?,

        GetContestsMode::User(user_id) => sqlx::query_as!(
            PublicContestConfig,
            r#"select
                    c.id,
                    c.owner_id,
                    users.login as owner_login,
                    c.name_ru as "name_ru!",
                    c.name_en as "name_en!",
                    c.statements_url_ru as "statements_url_ru!",
                    c.editorial_url_ru as "editorial_url_ru!",
                    c.statements_url_en as "statements_url_en!",
                    c.editorial_url_en as "editorial_url_en!",
                    c.starts_at,
                    c.finishes_at,
                    c.hidden,
                    c.upsolving_enabled,
                    c.solutions_hidden,
                    c.leaderboard_hidden,
                    c.created_at,
                    coalesce(
                        array_agg(co.user_id) filter (where co.user_id is not null),
                        '{}'
                    ) as "co_authors!" from contests c
                    left join contests_co_authors co on co.contest_id = c.id
                    left join users on users.id = c.owner_id
                    where c.owner_id = $1
                    group by c.id, owner_login
                    order by c.id desc"#,
            user_id,
        )
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
    sqlx::query_as!(
        PublicContestConfig,
        r#"select
                c.id,
                c.owner_id,
                users.login as owner_login,
                c.name_ru as "name_ru!",
                c.name_en as "name_en!",
                c.statements_url_ru as "statements_url_ru!",
                c.editorial_url_ru as "editorial_url_ru!",
                c.statements_url_en as "statements_url_en!",
                c.editorial_url_en as "editorial_url_en!",
                c.starts_at,
                c.finishes_at,
                c.hidden,
                c.upsolving_enabled,
                c.solutions_hidden,
                c.leaderboard_hidden,
                c.created_at,
                coalesce(
                    array_agg(co.user_id) filter (where co.user_id is not null),
                    '{}'
                ) as "co_authors!" from contests c
                left join contests_co_authors co on co.contest_id = c.id
                left join users on users.id = c.owner_id
                where c.id = $1
                group by c.id, owner_login"#,
        contest_id
    )
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

    let contest_id: i64 = sqlx::query_scalar!(
        r#"insert into contests
            (owner_id, name_ru, name_en, starts_at,
            finishes_at, statements_url_ru, editorial_url_ru, statements_url_en, editorial_url_en, hidden,
            upsolving_enabled, solutions_hidden, leaderboard_hidden) values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) returning id"#,
            user_id,
            &contest.name_ru,
            &contest.name_en,
            contest.starts_at,
            contest.finishes_at,
            &contest.statements_url_ru,
            &contest.editorial_url_ru,
            &contest.statements_url_en,
            &contest.editorial_url_en,
            contest.hidden,
            contest.upsolving_enabled,
            contest.solutions_hidden,
            contest.leaderboard_hidden,
        )
        .fetch_one(&mut *tx)
        .await
        .map_err(|_| AdaJudgeError::Internal)?;

    for user_id in &contest.co_authors {
        sqlx::query!(
            r#"insert into contests_co_authors (contest_id, user_id) values ($1, $2)"#,
            contest_id,
            user_id
        )
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

    sqlx::query!(r#"update contests set name_ru = $1, name_en = $2, starts_at = $3,
                    finishes_at = $4, statements_url_ru = $5, editorial_url_ru = $6, statements_url_en = $7,
                    editorial_url_en = $8, hidden = $9, upsolving_enabled = $10,
                    solutions_hidden = $11, leaderboard_hidden = $12 where id = $13"#,
                        &contest.name_ru,
                        &contest.name_en,
                        contest.starts_at,
                        contest.finishes_at,
                        &contest.statements_url_ru,
                        &contest.editorial_url_ru,
                        &contest.statements_url_en,
                        &contest.editorial_url_en,
                        contest.hidden,
                        contest.upsolving_enabled,
                        contest.solutions_hidden,
                        contest.leaderboard_hidden,
                        contest_id
            )
            .execute(&mut *tx)
            .await
            .map_err(|e| match e {
            sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
            _ => AdaJudgeError::Internal,
        })?;

    sqlx::query!(
        r#"delete from contests_co_authors where contest_id = $1"#,
        contest_id
    )
    .execute(&mut *tx)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
        _ => AdaJudgeError::Internal,
    })?;

    for user_id in &contest.co_authors {
        sqlx::query!(
            r#"insert into contests_co_authors (contest_id, user_id) values ($1, $2)"#,
            contest_id,
            user_id
        )
        .execute(&mut *tx)
        .await
        .map_err(|_| AdaJudgeError::Internal)?;
    }

    tx.commit().await.map_err(|_| AdaJudgeError::Internal)?;

    Ok(())
}

pub async fn delete_contest(pool: &PgPool, contest_id: i64) -> Result<(), AdaJudgeError> {
    sqlx::query!(r#"delete from contests where id = $1"#, contest_id)
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
    sqlx::query!(
        r#"insert into contests_posts (owner_id, contest_id, title_ru,
            text_ru, title_en, text_en) values ($1, $2, $3, $4, $5, $6)"#,
        user_id,
        contest_id,
        &post.title_ru,
        &post.text_ru,
        &post.title_en,
        &post.text_en
    )
    .execute(pool)
    .await
    .map_err(|_| AdaJudgeError::Internal)?;

    Ok(())
}

pub async fn update_contest_post(
    pool: &PgPool,
    post_id: i64,
    post: &ContestPostRequest,
) -> Result<(), AdaJudgeError> {
    sqlx::query!(
        r#"update contests_posts set title_ru = $1, text_ru = $2,
                    title_en = $3, text_en = $4 where id = $5"#,
        &post.title_ru,
        &post.text_ru,
        &post.title_en,
        &post.text_en,
        post_id
    )
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
    sqlx::query!(r#"delete from contests_posts where id = $1"#, post_id)
        .execute(pool)
        .await
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
            _ => AdaJudgeError::Internal,
        })?;

    Ok(())
}

pub async fn get_contest_post(pool: &PgPool, post_id: i64) -> Result<ContestPost, AdaJudgeError> {
    sqlx::query_as!(
        ContestPost,
        r#"select c.id as "id!",
        c.owner_id as "owner_id!",
        users.login as "owner_login",
        c.contest_id as "contest_id!",
        c.title_ru, c.title_en,
        c.text_ru, c.text_en, c.created_at
        from contests_posts c
        join users on users.id = c.owner_id
        where c.id = $1"#,
        post_id
    )
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
    let posts = sqlx::query_as!(
        ContestPost,
        r#"select c.id as "id!",
        c.owner_id as "owner_id!",
        users.login as "owner_login",
        c.contest_id as "contest_id!",
        c.title_ru, c.title_en,
        c.text_ru, c.text_en, c.created_at
        from contests_posts c
        join users on users.id = c.owner_id where c.contest_id = $1 order by c.id desc"#,
        contest_id
    )
    .fetch_all(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
        _ => AdaJudgeError::Internal,
    })?;

    Ok(posts)
}
