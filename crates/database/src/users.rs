//! Database tools for auth

use aj_models::{users::AdminLevel, verdicts::TestingVerdict};
use models::users::DatabaseUser;
use sqlx::PgPool;
use tools::map::MapLogExt;

/// Gets all users' ids
/// # Errors
/// Returns an error in case of internal error
pub async fn get_users(pool: &PgPool) -> Result<Vec<i64>, TestingVerdict> {
    sqlx::query_as::<_, (i64,)>("select id from users order by id")
        .fetch_all(pool)
        .await
        .map(|rows| rows.iter().map(|(id,)| *id).collect())
        .map_log(TestingVerdict::InvalidRequest)
}

/// Creates a user with login and password hash and returns it's id
/// # Errors
/// Returns an error if the user with this login exists
pub async fn create_user(
    pool: &PgPool,
    login: &str,
    password_hash: &str,
) -> Result<i64, TestingVerdict> {
    let user_id =
        sqlx::query_scalar("insert into users (login, password_hash) values ($1, $2) returning id")
            .bind(login)
            .bind(password_hash)
            .fetch_one(pool)
            .await
            .map_log(TestingVerdict::InvalidRequest)?;

    Ok(user_id)
}

/// Deletes a user by given id
/// # Errors
/// Returns an error if the user with this id does not exist
pub async fn delete_user(pool: &PgPool, id: i64) -> Result<(), TestingVerdict> {
    sqlx::query("delete from users where id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_log(TestingVerdict::InvalidRequest)?;

    Ok(())
}

/// Changes user's admin level
/// # Errors
/// Returns an error if the user with this id does not exist
pub async fn change_user_admin_level(
    pool: &PgPool,
    id: i64,
    admin_level: &AdminLevel,
) -> Result<(), TestingVerdict> {
    sqlx::query("update users set admin_level = $1 where id = $2")
        .bind(admin_level)
        .bind(id)
        .execute(pool)
        .await
        .map_log(TestingVerdict::InvalidRequest)?;

    Ok(())
}

/// Gets a user with target login
/// # Errors
/// Returns an error if the user with this login does not exist
pub async fn get_user_by_login(pool: &PgPool, login: &str) -> Result<DatabaseUser, TestingVerdict> {
    sqlx::query_as("select * from users where login = $1")
        .bind(login)
        .fetch_one(pool)
        .await
        .map_log(TestingVerdict::InvalidRequest)
}

/// Gets a user with target id
/// # Errors
/// Returns an error if the user with this id does not exist
pub async fn get_user_by_id(pool: &PgPool, id: i64) -> Result<DatabaseUser, TestingVerdict> {
    sqlx::query_as("select * from users where id = $1")
        .bind(id)
        .fetch_one(pool)
        .await
        .map_log(TestingVerdict::InvalidRequest)
}
