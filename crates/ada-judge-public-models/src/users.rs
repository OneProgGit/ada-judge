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

/// Delete account request called from frontend
#[derive(Clone, Debug, Deserialize)]
pub struct DeleteAccountRequest {
    /// Login
    pub login: String,
    /// Password
    pub password: String,
    /// Password confirmation
    pub password_confirmation: String,
    /// Deletion confirmation
    pub deletion_confirmation: bool,
}

/// Admin level
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize, PartialOrd, Ord)]
#[cfg_attr(not(target_arch = "wasm32"), derive(sqlx::Type))]
#[cfg_attr(
    not(target_arch = "wasm32"),
    sqlx(type_name = "admin_level", rename_all = "snake_case")
)]
#[serde(rename_all = "snake_case")]
pub enum AdminLevel {
    /// Not admin: can create private contests only
    NotAdmin,
    /// Beta tester: just a status
    BetaTester,
    /// Admin level I: TODO
    AdminI,
    /// Admin level II: can create contests and do things in contest in any time
    AdminII,
    /// Admin level III: TODO
    AdminIII,
    /// Owner: TODO
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
