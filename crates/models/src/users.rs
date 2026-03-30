//! Structs for users

use chrono::{DateTime, Utc};
use serde::Deserialize;

/// Register request called from frontend
#[derive(Clone, Debug, Deserialize)]
pub struct RegisterRequest {
    /// Login
    pub login: String,
    /// Password
    pub password: String,
    /// Password confirmation
    pub password_confirmation: String,
}

/// Login request called from frontend
pub struct LoginRequest {
    /// Login
    pub login: String,
    /// Password
    pub password: String,
}

/// Admin level
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
pub struct PublicUserData {
    /// Login
    pub login: String,
    /// Admin level
    pub admin_level: AdminLevel,
}

/// User data which is avaible only for user
pub struct PrivateUserData {
    /// Public user data
    pub public_user_data: PublicUserData,
    /// Timestamp when account was created
    pub created_at: DateTime<Utc>,
}

/// User data for database operations
pub struct DatabaseUser {
    /// Private user data
    pub private_user_data: PrivateUserData,
    /// Password hash
    pub password_hash: String,
}
