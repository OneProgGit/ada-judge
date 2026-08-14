use aj_models::problems::{ProblemTestingType, ProblemType, PublicProblemConfig, Subgroup};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;

#[derive(Deserialize, Serialize, Clone, Debug, sqlx::FromRow)]
pub struct DatabaseProblemConfig {
    pub id: i64,
    pub owner_id: Option<i64>,
    pub name_ru: String,
    pub name_en: String,
    pub r#type: ProblemType,
    pub testing_type: ProblemTestingType,
    pub contest_id: i64,
    pub index: i64,
    pub time_limit_ms: i32,
    pub memory_limit_mb: i32,
    pub checker_src_path: String,
    pub tests_path: String,
    pub subgroups: Json<Vec<Subgroup>>,
}

impl From<DatabaseProblemConfig> for PublicProblemConfig {
    fn from(value: DatabaseProblemConfig) -> Self {
        Self {
            id: value.id,
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
        }
    }
}
