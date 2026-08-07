//! Contests' config structs

use aj_models::contests::PublicContestConfig;
use chrono::{DateTime, Utc};

/// Contest's config for database
#[derive(Clone, Debug, sqlx::FromRow)]
pub struct DatabaseContestConfig {
    /// Contest's id
    pub id: i64,
    /// Contest's owner's user id (optional)
    pub owner_id: Option<i64>,
    /// Contest's name (ru)
    pub name_ru: String,
    /// Contest's name (en)
    pub name_en: String,
    /// Statements url (ru)
    pub statements_url_ru: String,
    /// Editorial url (ru)
    pub editorial_url_ru: String,
    /// Statements url (en)
    pub statements_url_en: String,
    /// Editorial url (en)
    pub editorial_url_en: String,
    /// Timestamp of contest beginning
    pub starts_at: DateTime<Utc>,
    /// Timestamp of contest ending
    pub ends_at: DateTime<Utc>,
    /// Timestamp of contest creating
    pub created_at: DateTime<Utc>,
    /// Is contest hidden
    pub hidden: bool,
    /// Is upsolving opened
    pub upsolving_opened: bool,
    /// Hide solutions' files
    pub hide_solutions: bool,
    /// Hide leaderboard till contest's finishing
    pub hide_leaderboard: bool,
    /// Contests' co-authors
    pub co_authors: Vec<i64>,
}

impl From<DatabaseContestConfig> for PublicContestConfig {
    fn from(value: DatabaseContestConfig) -> Self {
        Self {
            id: value.id,
            owner_id: value.owner_id,
            name_ru: value.name_ru,
            name_en: value.name_en,
            statements_url_ru: value.statements_url_ru,
            editorial_url_ru: value.editorial_url_ru,
            statements_url_en: value.statements_url_en,
            editorial_url_en: value.editorial_url_en,
            starts_at: value.starts_at,
            ends_at: value.ends_at,
            hidden: value.hidden,
            upsolving_opened: value.upsolving_opened,
            hide_solutions: value.hide_solutions,
            hide_leaderboard: value.hide_leaderboard,
            co_authors: value.co_authors,
        }
    }
}
