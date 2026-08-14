use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub login: String,
    pub password: String,
    pub password_confirmation: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub login: String,
    pub password: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct PublicUserData {
    pub id: i64,
    pub login: String,
    pub admin_level: AdminLevel,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PrivateUserData {
    pub id: i64,
    pub login: String,
    pub admin_level: AdminLevel,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, PartialOrd, Ord)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(
    feature = "sqlx",
    sqlx(type_name = "admin_level", rename_all = "snake_case")
)]
#[serde(rename_all = "snake_case")]
pub enum AdminLevel {
    User,
    Admin,
    Owner,
}
