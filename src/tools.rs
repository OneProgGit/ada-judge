use aj_models::users::AdminLevel;
use chrono::Utc;
use sqlx::PgPool;

pub fn is_allowed(user_id: i64, owner_id: Option<i64>, admin_level: &AdminLevel) -> bool {
    if admin_level == &AdminLevel::Owner {
        return true;
    }
    if owner_id.is_none() {
        return false;
    }
    if let Some(owner_id) = owner_id
        && owner_id != user_id
    {
        return false;
    }
    return true;
}

pub async fn check_contest_started_and_not_ended(
    pool: &PgPool,
    user_id: i64,
    contest_id: i64,
    problem_id: i64,
    admin_level: AdminLevel,
) -> bool {
    let Ok(contest) = database::contests::get_contest_by_id(pool, contest_id).await else {
        return false;
    };
    let Ok(problem) = database::problems::get_problem_by_id(pool, problem_id).await else {
        return false;
    };

    let now = Utc::now();

    return !((now < contest.starts_at
        || (now >= contest.ends_at && !contest.upsolving_opened)
        || contest.hidden)
        && !is_allowed(user_id, problem.owner_id, &admin_level)
        && !is_allowed(user_id, contest.owner_id, &admin_level));
}
