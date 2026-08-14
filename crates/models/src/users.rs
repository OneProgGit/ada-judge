use aj_models::users::{AdminLevel, PrivateUserData, PublicUserData};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug, sqlx::FromRow)]
pub struct DatabaseUser {
    pub id: i64,
    pub login: String,
    pub password_hash: String,
    pub admin_level: AdminLevel,
    pub created_at: DateTime<Utc>,
}

impl From<DatabaseUser> for PublicUserData {
    fn from(value: DatabaseUser) -> Self {
        Self {
            id: value.id,
            login: value.login,
            admin_level: value.admin_level,
        }
    }
}

impl From<DatabaseUser> for PrivateUserData {
    fn from(value: DatabaseUser) -> Self {
        Self {
            id: value.id,
            login: value.login,
            admin_level: value.admin_level,
            created_at: value.created_at,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub id: i64,
    pub exp: usize,
}
