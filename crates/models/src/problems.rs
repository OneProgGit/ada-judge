use aj_models::{
    problems::{ProblemConfig, ProblemTestingType, ProblemType, PublicProblemConfig, Subgroup},
    testing::Language,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;

#[derive(Deserialize, Serialize, Clone, Debug, sqlx::FromRow)]
pub struct DatabaseProblemConfig {
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
    pub checker_path: String,
    pub checker_lang: Language,
    pub tests_path: String,
    pub subgroups: Json<Vec<Subgroup>>,
    pub created_at: DateTime<Utc>,
}

impl From<DatabaseProblemConfig> for PublicProblemConfig {
    fn from(value: DatabaseProblemConfig) -> Self {
        Self {
            id: value.id,
            owner_id: value.owner_id,
            owner_login: value.owner_login,
            r#type: value.r#type,
            testing_type: value.testing_type,
            contest_id: value.contest_id,
            index: value.index,
            name_ru: value.name_ru,
            name_en: value.name_en,
            time_limit_ms: value.time_limit_ms,
            memory_limit_mb: value.memory_limit_mb,
            subgroups: value.subgroups.0,
            created_at: value.created_at,
        }
    }
}

impl From<DatabaseProblemConfig> for ProblemConfig {
    fn from(value: DatabaseProblemConfig) -> Self {
        Self {
            owner_id: value.owner_id,
            r#type: value.r#type,
            testing_type: value.testing_type,
            contest_id: value.contest_id,
            index: value.index,
            name_ru: value.name_ru,
            name_en: value.name_en,
            time_limit_ms: value.time_limit_ms,
            memory_limit_mb: value.memory_limit_mb,
            subgroups: value.subgroups.0,
            checker_path: value.checker_path,
            checker_lang: value.checker_lang,
            tests_path: value.tests_path,
        }
    }
}
