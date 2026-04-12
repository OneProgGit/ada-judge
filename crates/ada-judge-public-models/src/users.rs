//! Structs for users

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Register request called from frontend
#[derive(Clone, Debug, Deserialize)]
pub struct RegisterRequest {
    /// Login
    pub login: String,
    /// Password
    pub password: String,
    /// Password confirmation
    pub password_confirmation: String,
    /// Master password (only for now)
    pub master_password: String,
}

/// Login request called from frontend
#[derive(Clone, Debug, Deserialize)]
pub struct LoginRequest {
    /// Login
    pub login: String,
    /// Password
    pub password: String,
}

/// Admin level
#[derive(Clone, PartialEq, Eq, sqlx::Type, Debug, Serialize, Deserialize, PartialOrd, Ord)]
#[sqlx(type_name = "admin_level", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum AdminLevel {
    /// Not admin: can create private contests only
    NotAdmin,
    /// Admin level I: can be a co-author of public contest
    AdminI,
    /// Admin level II: can create public contests (with Admin level III+ moderation)
    AdminII,
    /// Admin level III: can moderate public contests
    AdminIII,
    /// Owner: can manage all public contests and system settings
    Owner,
}

/// User data which is avaible for all users
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct PublicUserData {
    /// User id
    pub id: i64,
    /// Login
    pub login: String,
    /// Admin level
    pub admin_level: AdminLevel,
}

/// User data which is avaible only for user
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct PrivateUserData {
    /// User id
    pub id: i64,
    /// Login
    pub login: String,
    /// Admin level
    pub admin_level: AdminLevel,
    /// Timestamp when account was created
    pub created_at: DateTime<Utc>,
}
