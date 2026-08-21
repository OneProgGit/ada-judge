use aj_models::{
    errors::{AdaJudgeError, AuthError},
    users::{AdminLevel, PrivateUserData},
};
use models::users::DatabaseUser;
use sqlx::PgPool;

pub async fn get_users(pool: &PgPool) -> Result<Vec<PrivateUserData>, AdaJudgeError> {
    let users = sqlx::query_as!(
        DatabaseUser,
        r#"select id,
            login,
            password_hash,
            admin_level as "admin_level: AdminLevel",
            created_at
            from users order by id"#,
    )
    .fetch_all(pool)
    .await
    .map_err(|_| AdaJudgeError::Internal)?
    .iter()
    .map(|x| x.clone().into())
    .collect();

    Ok(users)
}

pub async fn create_user(
    pool: &PgPool,
    login: &str,
    password_hash: &str,
) -> Result<i64, AdaJudgeError> {
    let user_id = sqlx::query_scalar!(
        r#"insert into users (login, password_hash) values ($1, $2) returning id"#,
        login,
        password_hash
    )
    .fetch_one(pool)
    .await
    .map_err(|e| {
        if e.as_database_error()
            .is_some_and(|e| e.is_unique_violation())
        {
            AdaJudgeError::Auth(AuthError::AlreadyExists)
        } else {
            AdaJudgeError::Internal
        }
    })?;

    Ok(user_id)
}

pub async fn delete_user(pool: &PgPool, id: i64) -> Result<(), AdaJudgeError> {
    sqlx::query!(r#"delete from users where id = $1"#, id)
        .execute(pool)
        .await
        .map_err(|_| AdaJudgeError::Internal)?;

    Ok(())
}

pub async fn change_admin_level(
    pool: &PgPool,
    id: i64,
    admin_level: &AdminLevel,
) -> Result<(), AdaJudgeError> {
    sqlx::query!(
        r#"update users set admin_level = $1 where id = $2"#,
        admin_level.clone() as AdminLevel,
        id
    )
    .execute(pool)
    .await
    .map_err(|_| AdaJudgeError::Internal)?;

    Ok(())
}

pub async fn get_user_by_login(pool: &PgPool, login: &str) -> Result<DatabaseUser, AdaJudgeError> {
    sqlx::query_as!(
        DatabaseUser,
        r#"select id,
        login,
        password_hash,
        admin_level as "admin_level: AdminLevel",
        created_at from users where login = $1"#,
        login
    )
    .fetch_one(pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => AdaJudgeError::NotFound,
        _ => AdaJudgeError::Internal,
    })
}

pub async fn get_user_by_id(pool: &PgPool, id: i64) -> Result<DatabaseUser, AdaJudgeError> {
    sqlx::query_as!(
        DatabaseUser,
        r#"select id,
        login,
        password_hash,
        admin_level as "admin_level: AdminLevel",
        created_at from users where id = $1"#,
        id
    )
    .fetch_one(pool)
    .await
    .map_err(|_| AdaJudgeError::Internal)
}
