//! Structs for users

use ada_judge_public_models::users::{AdminLevel, PrivateUserData, PublicUserData};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// User data for database operations
#[derive(Deserialize, Serialize, Clone, Debug, sqlx::FromRow)]
pub struct DatabaseUser {
    /// User id
    pub id: i64,
    /// Login
    pub login: String,
    /// Admin level
    pub admin_level: AdminLevel,
    /// Timestamp when account was created
    pub created_at: DateTime<Utc>,
    /// Password hash
    pub password_hash: String,
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

/// Json web token claims
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    /// User id
    pub id: i64,
    /// Expire datetime
    pub exp: usize,
}
