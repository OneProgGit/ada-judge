//! Database tools for auth

use ada_judge_public_models::verdicts::TotalVerdict;
use models::users::DatabaseUser;
use sqlx::PgPool;
use tools::map::MapLogExt;

/// Creates a user with login and password hash and returns it's id
/// # Errors
/// Returns an error if the user with this login exists
pub async fn create_user(
    pool: &PgPool,
    login: &str,
    password_hash: &str,
) -> Result<i64, TotalVerdict> {
    let user_id =
        sqlx::query_scalar("insert into users (login, password_hash) values ($1, $2) returning id")
            .bind(login)
            .bind(password_hash)
            .fetch_one(pool)
            .await
            .map_log(TotalVerdict::InvalidRequest)?;

    Ok(user_id)
}

/// Deletes a user by given id
/// # Errors
/// Returns an error if the user with this id does not exist
pub async fn delete_user(pool: &PgPool, id: i64) -> Result<(), TotalVerdict> {
    sqlx::query("delete from users where id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_log(TotalVerdict::InvalidRequest)?;

    Ok(())
}

/// Gets a user with target login
/// # Errors
/// Returns an error if the user with this login does not exist
pub async fn get_user_by_login(pool: &PgPool, login: &str) -> Result<DatabaseUser, TotalVerdict> {
    sqlx::query_as("select * from users where login = $1")
        .bind(login)
        .fetch_one(pool)
        .await
        .map_log(TotalVerdict::InvalidRequest)
}

/// Gets a user with target id
/// # Errors
/// Returns an error if the user with this id does not exist
pub async fn get_user_by_id(pool: &PgPool, id: i64) -> Result<DatabaseUser, TotalVerdict> {
    sqlx::query_as("select * from users where id = $1")
        .bind(id)
        .fetch_one(pool)
        .await
        .map_log(TotalVerdict::InvalidRequest)
}
