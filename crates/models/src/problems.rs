//! Problem's config structs

use ada_judge_public_models::problems::{
    ProblemConfig, ProblemType, PublicProblemConfig, Subgroup,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::types::Json;

/// Problem's config for database
#[derive(Deserialize, Serialize, Clone, Debug, sqlx::FromRow)]
pub struct DatabaseProblemConfig {
    /// Problem's id
    pub id: i64,
    /// Problem's owner id (optional)
    pub owner_id: Option<i64>,
    /// Problem's type
    pub r#type: ProblemType,
    /// Merge subgroups
    pub merge_subgroups: bool,
    /// Problems's contest id
    pub contest_id: i64,
    /// Problem's index in contest
    pub problem_index: i64,
    /// Problem's name (ru)
    pub name_ru: String,
    /// Problem's name (en)
    pub name_en: String,
    /// Testing time limit in milliseconds
    pub time_limit_ms: i32,
    /// Testing memory limit in megabytes
    pub memory_limit_mb: i32,
    /// Path to the checker relative to problem's path
    pub checker_path: String,
    /// Path to the directory, which contains directories with tests inputs and outputs, relative to problem's path
    pub tests_path: String,
    /// Testing subgroups
    pub subgroups: Json<Vec<Subgroup>>,
    /// Created at timestamp
    pub created_at: DateTime<Utc>,
}

impl From<DatabaseProblemConfig> for ProblemConfig {
    fn from(value: DatabaseProblemConfig) -> Self {
        Self {
            owner_id: value.owner_id,
            r#type: value.r#type,
            merge_subgroups: value.merge_subgroups,
            contest_id: value.contest_id,
            problem_index: value.problem_index,
            name_ru: value.name_ru,
            name_en: value.name_en,
            time_limit_ms: value.time_limit_ms,
            memory_limit_mb: value.memory_limit_mb,
            checker_path: value.checker_path,
            tests_path: value.tests_path,
            subgroups: value.subgroups.0,
        }
    }
}

impl From<DatabaseProblemConfig> for PublicProblemConfig {
    fn from(value: DatabaseProblemConfig) -> Self {
        Self {
            id: value.id,
            owner_id: value.owner_id,
            r#type: value.r#type,
            merge_subgroups: value.merge_subgroups,
            contest_id: value.contest_id,
            problem_index: value.problem_index,
            name_ru: value.name_ru,
            name_en: value.name_en,
            time_limit_ms: value.time_limit_ms,
            memory_limit_mb: value.memory_limit_mb,
            subgroups: value.subgroups.0,
        }
    }
}
