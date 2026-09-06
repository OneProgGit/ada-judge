use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::problems::{ProblemQuestion, PublicProblemConfig};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct LeaderboardRow {
    pub user_id: i64,
    pub user_login: String,
    pub scores: Vec<f64>,
    pub total_score: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(clippy::struct_excessive_bools)]
pub struct ContestRequest {
    pub name_ru: String,
    pub name_en: String,
    pub starts_at: DateTime<Utc>,
    pub finishes_at: DateTime<Utc>,
    pub statements_url_ru: String,
    pub statements_url_en: String,
    pub editorial_url_ru: String,
    pub editorial_url_en: String,
    pub hidden: bool,
    pub upsolving_enabled: bool,
    pub solutions_hidden: bool,
    pub leaderboard_hidden: bool,
    pub co_authors: Vec<i64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
#[allow(clippy::struct_excessive_bools)]
pub struct PublicContestConfig {
    pub id: i64,
    pub owner_id: Option<i64>,
    pub owner_login: Option<String>,
    pub name_ru: String,
    pub name_en: String,
    pub starts_at: DateTime<Utc>,
    pub finishes_at: DateTime<Utc>,
    pub statements_url_ru: String,
    pub statements_url_en: String,
    pub editorial_url_ru: String,
    pub editorial_url_en: String,
    pub hidden: bool,
    pub upsolving_enabled: bool,
    pub solutions_hidden: bool,
    pub leaderboard_hidden: bool,
    pub co_authors: Vec<i64>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContestPostRequest {
    pub title_ru: String,
    pub title_en: String,
    pub text_ru: String,
    pub text_en: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct ContestPost {
    pub id: i64,
    pub owner_id: i64,
    pub owner_login: String,
    pub contest_id: i64,
    pub title_ru: String,
    pub title_en: String,
    pub text_ru: String,
    pub text_en: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum ContestEvent {
    NewPost(ContestPost),
    PostUpdated(ContestPost),
    PostDeleted(i64),
    ContestUpdated(PublicContestConfig),
    ContestDeleted,
    NewProblem(PublicProblemConfig),
    ProblemUpdated(PublicProblemConfig),
    ProblemDeleted(i64),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum QuestionEvent {
    NewProblemQuestion(ProblemQuestion),
    ProblemQuestionDeleted(i64),
    ProblemQuestionAnswered(String),
}
