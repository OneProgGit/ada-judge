use serde::{Deserialize, Serialize};
use sqlx::types::Json;

/// Problem's config
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ProblemConfig {
    pub name: String,
    pub time_limit_ms: i32,
    pub memory_limit_mb: i32,
    pub checker_path: String,
    pub tests_path: String,
    pub subgroups: Vec<Subgroup>,
}

/// Problem's config for database
#[derive(Deserialize, Serialize, Clone, Debug, sqlx::FromRow)]
pub struct DatabaseProblemConfig {
    pub name: String,
    pub time_limit_ms: i32,
    pub memory_limit_mb: i32,
    pub checker_path: String,
    pub tests_path: String,
    pub subgroups: Json<Vec<Subgroup>>,
}

impl From<&ProblemConfig> for DatabaseProblemConfig {
    fn from(value: &ProblemConfig) -> Self {
        Self {
            name: value.name.clone(),
            time_limit_ms: value.time_limit_ms,
            memory_limit_mb: value.memory_limit_mb,
            checker_path: value.checker_path.clone(),
            tests_path: value.tests_path.clone(),
            subgroups: Json(value.subgroups.clone()),
        }
    }
}

impl From<DatabaseProblemConfig> for ProblemConfig {
    fn from(value: DatabaseProblemConfig) -> Self {
        Self {
            name: value.name,
            time_limit_ms: value.time_limit_ms,
            memory_limit_mb: value.memory_limit_mb,
            checker_path: value.checker_path,
            tests_path: value.tests_path,
            subgroups: value.subgroups.0,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, sqlx::FromRow)]
pub struct Subgroup {
    pub r#type: SubgroupType,
    pub tests: Vec<i32>,
    pub score: i32,
    pub depends_on: Vec<usize>,
}

/// Test group's type
#[derive(Deserialize, Serialize, Debug, Clone, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "subgroup_type", rename_all = "snake_case")]
pub enum SubgroupType {
    Sample,
    Main,
}
