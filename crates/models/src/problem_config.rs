//! Problem's config structs

use serde::{Deserialize, Serialize};
use sqlx::types::Json;

/// Problem's config
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ProblemConfig {
    /// Problems's contest id
    pub contest_id: i32,
    /// Problem's name or title
    pub name: String,
    /// Testing time limit in milliseconds
    pub time_limit_ms: i32,
    /// Testing memory limit in megabytes
    pub memory_limit_mb: i32,
    /// Path to the checker relative to problem's path
    pub checker_path: String,
    /// Path to the directory, which contains directories with tests inputs and outputs, relative to problem's path
    pub tests_path: String,
    /// Testing subgroups
    pub subgroups: Vec<Subgroup>,
}

/// Problem's config for database
#[derive(Deserialize, Serialize, Clone, Debug, sqlx::FromRow)]
pub struct DatabaseProblemConfig {
    /// Problems's contest id
    pub contest_id: i32,
    /// Problem's name or title
    pub name: String,
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
}

impl From<&ProblemConfig> for DatabaseProblemConfig {
    fn from(value: &ProblemConfig) -> Self {
        Self {
            contest_id: value.contest_id,
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
            contest_id: value.contest_id,
            name: value.name,
            time_limit_ms: value.time_limit_ms,
            memory_limit_mb: value.memory_limit_mb,
            checker_path: value.checker_path,
            tests_path: value.tests_path,
            subgroups: value.subgroups.0,
        }
    }
}

/// Testing subgroup
#[derive(Deserialize, Serialize, Debug, Clone, sqlx::FromRow)]
pub struct Subgroup {
    /// Subgroup's type
    pub r#type: SubgroupType,
    /// Array of tests' indexes of the subgroup
    pub tests: Vec<i32>,
    /// Maximum score which can be obtained from the subgroup
    pub score: i32,
    /// Indexes of the subgroups, all of them must have `Ok` verdict to test on this subgroup.
    /// Also, they must be less than index of this subgroup
    pub depends_on: Vec<usize>,
}

/// Subgroup's type
#[derive(Deserialize, Serialize, Debug, Clone, sqlx::Type)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "subgroup_type", rename_all = "snake_case")]
pub enum SubgroupType {
    /// Don't count score for this subgroup
    Sample,
    /// Count score for this subgroup
    Main,
}
