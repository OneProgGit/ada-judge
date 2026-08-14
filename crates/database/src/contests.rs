use aj_models::{
    contests::{ContestPost, LeaderboardRow},
    errors::Error,
    problems::PublicProblemConfig,
    verdicts::TestingVerdict,
};
use models::contests::DatabaseContestConfig;
use sqlx::{
    PgPool,
    postgres::PgSeverity::Error,
    types::chrono::{DateTime, Utc},
};
use tools::map::MapLogExt;

pub struct ContestsRepository {
    pool: Arc<PgPool>,
}

pub enum GetContestsMode {
    User(i64),
    NotHidden(i64),
    All,
}

impl ContestsRepository {
    pub async fn get_leaderboard(&self, contest_id: i64) -> Result<Vec<LeaderboardRow>, Error> {
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
                    select distinct user_id
                    from submissions s
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
        .map_err(Error::Internal)?;

        Ok(leaderboard)
    }

    pub async fn get_problems(&self, contest_id: i64) -> Result<Vec<PublicProblemConfig>, Error> {
        sqlx::query_as(
            "select
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
                where c.contest_id = $1 order by index",
        )
        .bind(contest_id)
        .fetch_all(pool)
        .await
        .map(|rows| rows.iter().map(|(x,)| *x).collect())
        .map_err(Error::Internal)
    }

    pub async fn get_contests(&self, mode: GetContestsMode) -> Result<Vec<i64>, TestingVerdict> {
        match mode {
            GetContestsMode::All => {
                sqlx::query_as::<_, (i64,)>("select id from contests order by id desc")
                    .fetch_all(pool)
                    .await
                    .map(|rows| rows.iter().map(|(id,)| *id).collect())
                    .map_log(TestingVerdict::InvalidRequest)
            }

            GetContestsMode::NotHidden(user_id) => sqlx::query_as::<_, (i64,)>(
                "select id from contests where not hidden or owner_id = $1
                    or exists(
                        select 1 from contests_co_authors
                        where contest_id = contests.id
                            and user_id = $1
                    ) order by id desc",
            )
            .bind(user_id)
            .fetch_all(pool)
            .await
            .map(|rows| rows.iter().map(|(id,)| *id).collect())
            .map_log(TestingVerdict::InvalidRequest),

            GetContestsMode::User(user_id) => sqlx::query_as::<_, (i64,)>(
                "select id from contests where owner_id = $1 order by id desc",
            )
            .bind(user_id)
            .fetch_all(pool)
            .await
            .map(|rows| rows.iter().map(|(id,)| *id).collect())
            .map_log(TestingVerdict::InvalidRequest),
        }
    }

    pub async fn get_contest(
        &self,
        contest_id: i64,
    ) -> Result<DatabaseContestConfig, TestingVerdict> {
        sqlx::query_as(
            "select
                    c.id,
                    c.owner_id,
                    c.name_ru,
                    c.name_en,
                    c.statements_url_ru,
                    c.editorial_url_ru,
                    c.statements_url_en,
                    c.editorial_url_en,
                    c.starts_at,
                    c.ends_at,
                    c.created_at,
                    c.hidden,
                    c.upsolving_opened,
                    c.hide_solutions,
                    c.hide_leaderboard,
                    coalesce(
                        array_agg(co.user_id) filter (where co.user_id is not null),
                        '{}'
                    ) as co_authors from contests c
                    left join contests_co_authors co on co.contest_id = c.id
                    where c.id = $1
                    group by c.id",
        )
        .bind(contest_id)
        .fetch_one(pool)
        .await
        .map_log(TestingVerdict::InvalidRequest)
    }

    /// Creates a contest by given contest data
    /// # Errors
    /// Returns an error if `owner_id` is invalid
    pub async fn create_contest(
        pool: &PgPool,
        owner_id: i64,
        name_ru: &str,
        name_en: &str,
        starts_at: &DateTime<Utc>,
        ends_at: &DateTime<Utc>,
        statements_url_ru: &str,
        editorial_url_ru: &str,
        statements_url_en: &str,
        editorial_url_en: &str,
        hidden: bool,
        upsolving_opened: bool,
        hide_solutions: bool,
        hide_leaderboard: bool,
    ) -> Result<i64, TestingVerdict> {
        let contest_id = sqlx::query_scalar(
            "insert into contests
                (owner_id, name_ru, name_en, starts_at,
                ends_at, statements_url_ru, editorial_url_ru, statements_url_en, editorial_url_en, hidden, upsolving_opened,
                hide_solutions, hide_leaderboard) values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13) returning id",
        )
        .bind(owner_id)
        .bind(name_ru)
        .bind(name_en)
        .bind(starts_at)
        .bind(ends_at)
        .bind(statements_url_ru)
        .bind(editorial_url_ru)
        .bind(statements_url_en)
        .bind(editorial_url_en)
        .bind(hidden)
        .bind(upsolving_opened)
        .bind(hide_solutions)
        .bind(hide_leaderboard)
        .fetch_one(pool)
        .await
        .map_log(TestingVerdict::InvalidRequest)?;

        Ok(contest_id)
    }

    /// Updates a contest by given contest id and contest data
    /// # Errors
    /// Returns an error if `contest_id` is invalid
    pub async fn update_contest(
        pool: &PgPool,
        contest_id: i64,
        name_ru: &str,
        name_en: &str,
        starts_at: &DateTime<Utc>,
        ends_at: &DateTime<Utc>,
        statements_url_ru: &str,
        editorial_url_ru: &str,
        statements_url_en: &str,
        editorial_url_en: &str,
        hidden: bool,
        upsolving_opened: bool,
        hide_solutions: bool,
        hide_leaderboard: bool,
    ) -> Result<(), TestingVerdict> {
        sqlx::query("update contests set name_ru = $1, name_en = $2, starts_at = $3,
                    ends_at = $4, statements_url_ru = $5, editorial_url_ru = $6, statements_url_en = $7, editorial_url_en = $8, hidden = $9, upsolving_opened = $10,
                    hide_solutions = $11, hide_leaderboard = $12 where id = $13")
            .bind(name_ru)
            .bind(name_en)
            .bind(starts_at)
            .bind(ends_at)
            .bind(statements_url_ru)
            .bind(editorial_url_ru)
            .bind(statements_url_en)
            .bind(editorial_url_en)
            .bind(hidden)
            .bind(upsolving_opened)
            .bind(hide_solutions)
            .bind(hide_leaderboard)
            .bind(contest_id)
            .execute(pool)
            .await
            .map_log(TestingVerdict::InvalidRequest)?;

        Ok(())
    }

    /// Erases and inserts contest's co-authors
    /// # Errors
    /// Returns an error if `contest_id` is invalid
    pub async fn insert_contest_co_authors(
        pool: &PgPool,
        contest_id: i64,
        co_authors: &Vec<i64>,
    ) -> Result<(), TestingVerdict> {
        let mut tx = pool.begin().await.map_log(TestingVerdict::Bug)?;
        sqlx::query("delete from contests_co_authors where contest_id = $1")
            .bind(contest_id)
            .execute(&mut *tx)
            .await
            .map_log(TestingVerdict::Bug)?;
        for co_author in co_authors {
            sqlx::query("insert into contests_co_authors (contest_id, user_id) values ($1, $2)")
                .bind(contest_id)
                .bind(co_author)
                .execute(&mut *tx)
                .await
                .map_log(TestingVerdict::Bug)?;
        }
        tx.commit().await.map_log(TestingVerdict::Bug)?;

        Ok(())
    }

    /// Deletes a contest by given id
    /// # Errors
    /// Returns an error if the contest with this id does not exist
    pub async fn delete_contest(pool: &PgPool, contest_id: i64) -> Result<(), TestingVerdict> {
        sqlx::query("delete from contests where id = $1")
            .bind(contest_id)
            .execute(pool)
            .await
            .map_log(TestingVerdict::InvalidRequest)?;

        Ok(())
    }

    /// Creates a post in contest by given post data
    /// # Errors
    /// Returns an error if `owner_id` is invalid
    pub async fn create_contest_post(
        pool: &PgPool,
        owner_id: i64,
        contest_id: i64,
        title_ru: &str,
        text_ru: &str,
        title_en: &str,
        text_en: &str,
    ) -> Result<i64, TestingVerdict> {
        let post_id = sqlx::query_scalar(
            "insert into contests_posts (owner_id, contest_id, title_ru, text_ru, title_en, text_en) values ($1, $2, $3, $4, $5, $6) returning id",
        )
        .bind(owner_id)
        .bind(contest_id)
        .bind(title_ru)
        .bind(text_ru)
        .bind(title_en)
        .bind(text_en)
        .fetch_one(pool)
        .await
        .map_log(TestingVerdict::InvalidRequest)?;

        Ok(post_id)
    }

    /// Updates a post in contest by given post data
    /// # Errors
    /// Returns an error if `post_id` is invalid
    pub async fn update_contest_post(
        pool: &PgPool,
        post_id: i64,
        title_ru: &str,
        text_ru: &str,
        title_en: &str,
        text_en: &str,
    ) -> Result<(), TestingVerdict> {
        sqlx::query("update contests_posts set title_ru = $1, text_ru = $2, title_en = $3, text_en = $4 where id = $5")
            .bind(title_ru)
            .bind(text_ru)
            .bind(title_en)
            .bind(text_en)
            .bind(post_id)
            .execute(pool)
            .await
            .map_log(TestingVerdict::InvalidRequest)?;

        Ok(())
    }

    /// Deletes a post from contest
    /// # Errors
    /// Returns an error if `post_id` is invalid
    pub async fn delete_contest_post(pool: &PgPool, post_id: i64) -> Result<(), TestingVerdict> {
        sqlx::query("delete from contests_posts where id = $1")
            .bind(post_id)
            .execute(pool)
            .await
            .map_log(TestingVerdict::InvalidRequest)?;

        Ok(())
    }

    /// Gets a contest's post by given id
    /// # Errors
    /// Returns an error if `post_id` is invalid
    pub async fn get_contest_post_by_id(
        pool: &PgPool,
        post_id: i64,
    ) -> Result<ContestPost, TestingVerdict> {
        sqlx::query_as("select * from contests_posts where id = $1")
            .bind(post_id)
            .fetch_one(pool)
            .await
            .map_log(TestingVerdict::InvalidRequest)
    }

    /// Gets a contest's posts
    /// # Errors
    /// Returns an error if `contest_id` is invalid
    pub async fn get_contest_posts(
        pool: &PgPool,
        contest_id: i64,
    ) -> Result<Vec<i64>, TestingVerdict> {
        sqlx::query_as::<_, (i64,)>(
            "select id from contests_posts where contest_id = $1 order by id desc",
        )
        .bind(contest_id)
        .fetch_all(pool)
        .await
        .map(|rows| rows.iter().map(|(id,)| *id).collect())
        .map_log(TestingVerdict::InvalidRequest)
    }
}
