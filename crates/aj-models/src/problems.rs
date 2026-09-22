use crate::{
    testing::{Language, SubgroupResult},
    verdicts::Verdict,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ProblemConfig {
    pub owner_id: Option<i64>,
    pub name_ru: String,
    pub name_en: String,
    pub r#type: ProblemType,
    pub testing_type: ProblemTestingType,
    pub contest_id: i64,
    pub index: i64,
    pub time_limit_ms: i32,
    pub memory_limit_mb: i32,
    pub checker_path: String,
    pub checker_lang: Language,
    pub tests_path: String,
    pub subgroups: Vec<Subgroup>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct PublicProblemConfig {
    pub id: i64,
    pub owner_id: Option<i64>,
    pub owner_login: Option<String>,
    pub name_ru: String,
    pub name_en: String,
    pub r#type: ProblemType,
    pub testing_type: ProblemTestingType,
    pub contest_id: i64,
    pub index: i64,
    pub time_limit_ms: i32,
    pub memory_limit_mb: i32,
    pub subgroups: Vec<Subgroup>,
    pub created_at: DateTime<Utc>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(
    feature = "sqlx",
    sqlx(type_name = "problem_type", rename_all = "snake_case")
)]
#[serde(rename_all = "snake_case")]
pub enum ProblemType {
    Default,
    Interactive,
    RunTwice,
    InteractiveRunTwice,
    RunTwiceFirstInteractive,
    RunTwiceSecondInteractive,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(
    feature = "sqlx",
    sqlx(type_name = "problem_testing_type", rename_all = "snake_case")
)]
#[serde(rename_all = "snake_case")]
pub enum ProblemTestingType {
    Ioi,
    IoiMergeSubgroups,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct Subgroup {
    pub r#type: SubgroupType,
    pub tests: Vec<i32>,
    pub score: Option<f64>,
    pub score_per_test: Option<f64>,
    pub depends_on: Vec<usize>,
}

impl Subgroup {
    #[must_use]
    pub fn should_skip(&self, subgroups_results: &[SubgroupResult]) -> bool {
        !self
            .depends_on
            .iter()
            .all(|i| subgroups_results[*i].verdict == Verdict::Ok)
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(
    feature = "sqlx",
    sqlx(type_name = "subgroup_type", rename_all = "snake_case")
)]
#[serde(rename_all = "snake_case")]
pub enum SubgroupType {
    Sample,
    Main,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProblemQuestionRequest {
    pub title: String,
    pub text: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct ProblemQuestion {
    pub id: i64,
    pub owner_id: i64,
    pub owner_login: String,
    pub problem_id: i64,
    pub title: String,
    pub text: String,
    pub answer: String,
    pub created_at: DateTime<Utc>,
}
