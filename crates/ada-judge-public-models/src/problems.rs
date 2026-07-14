//! Structs for problems

use serde::{Deserialize, Serialize};

/// Problem's config
#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ProblemConfig {
    /// Problem's owner's user id (optional)
    pub owner_id: Option<i64>,
    /// Problem's type
    pub r#type: ProblemType,
    /// Problems's contest id
    pub contest_id: i64,
    /// Problem's index in contest
    pub problem_index: i64,
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

/// Problem config visible for all users
#[derive(Deserialize, Serialize, Clone, Debug)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct PublicProblemConfig {
    /// Problem's id
    pub id: i64,
    /// Problem's owner id (optional)
    pub owner_id: Option<i64>,
    /// Problem's type
    pub r#type: ProblemType,
    /// Problems's contest id
    pub contest_id: i64,
    /// Problem's index in contest
    pub problem_index: i64,
    /// Problem's name or title
    pub name: String,
    /// Testing time limit in milliseconds
    pub time_limit_ms: i32,
    /// Testing memory limit in megabytes
    pub memory_limit_mb: i32,
    /// Problem's subgroups
    pub subgroups: Vec<Subgroup>,
}

/// Problem's type
#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(
    feature = "sqlx",
    sqlx(type_name = "problem_type", rename_all = "snake_case")
)]
#[serde(rename_all = "snake_case")]
pub enum ProblemType {
    /// Default problem
    Default,
    /// Interactive problem
    Interactive,
    /// Run-twice problem
    RunTwice,
}

/// Testing subgroup
#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct Subgroup {
    /// Subgroup's type
    pub r#type: SubgroupType,
    /// Array of tests' indexes of the subgroup
    pub tests: Vec<i32>,
    /// Maximum score which can be obtained from the subgroup
    pub score: Option<i32>,
    /// Maximum score which can be obtained from the each of subgroup's tests
    pub score_per_test: Option<i32>,
    /// Indexes of the subgroups, all of them must have `Ok` verdict to test on this subgroup.
    /// Also, they must be less than index of this subgroup
    pub depends_on: Vec<usize>,
}

/// Subgroup's type
#[derive(Deserialize, Serialize, Debug, Clone)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(
    feature = "sqlx",
    sqlx(type_name = "subgroup_type", rename_all = "snake_case")
)]
#[serde(rename_all = "snake_case")]
pub enum SubgroupType {
    /// Don't count score for this subgroup
    Sample,
    /// Count score for this subgroup
    Main,
}
