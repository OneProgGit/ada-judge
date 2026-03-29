use serde::Deserialize;
use std::path::PathBuf;

/// Problem's config
#[derive(Deserialize, Clone, sqlx::FromRow)]
pub struct ProblemConfig {
    pub name: String,
    pub time_limit_ms: u64,
    pub memory_limit_mb: u64,
    pub checker_path: PathBuf,
    pub tests_path: PathBuf,
    pub subgroups: Vec<Subgroup>,
}

#[derive(Deserialize, Clone, sqlx::FromRow)]
pub struct Subgroup {
    pub r#type: SubgroupType,
    pub tests: Vec<i32>,
    pub score: i32,
    pub depends_on: Vec<usize>,
}

/// Test group's type
#[derive(Deserialize, Clone, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "subgroup_type", rename_all = "snake_case")]
pub enum SubgroupType {
    Sample,
    Main,
}
