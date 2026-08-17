use aj_models::users::AdminLevel;
use chrono::Utc;
use sqlx::PgPool;

pub fn is_allowed(user_id: i64, owner_id: Option<i64>, admin_level: &AdminLevel) -> bool {
    owner_id.is_some_and(|owner_id| owner_id == user_id) || admin_level == &AdminLevel::Owner
}

pub async fn is_contest_active(
    pool: &PgPool,
    user_id: i64,
    contest_id: i64,
    problem_id: i64,
    admin_level: AdminLevel,
) -> bool {
    let Ok(contest) = database::contests::get_contest(pool, contest_id).await else {
        return false;
    };
    let Ok(problem) = database::problems::get_problem(pool, problem_id).await else {
        return false;
    };

    let now = Utc::now();

    return (now >= contest.starts_at
        && (now <= contest.finishes_at || contest.upsolving_enabled)
        && !contest.hidden)
        || is_allowed(user_id, problem.owner_id, &admin_level)
        || is_allowed(user_id, contest.owner_id, &admin_level)
        || contest.co_authors.binary_search(&user_id).is_ok();
}
