use std::path::PathBuf;

use serde::Deserialize;

/// Problem's config
#[derive(Deserialize)]
pub struct ProblemConfig {
    pub general: General,
    pub limits: Limits,
    pub checker: Checker,
    pub tests: Tests,
    pub test_groups: Vec<TestGroup>,
}

/// General data
#[derive(Deserialize)]
pub struct General {
    pub name: String,
}

/// Limits
#[derive(Deserialize)]
pub struct Limits {
    pub time_limit_ms: u64,
    pub memory_limit_mb: u64,
}

/// Checker info
#[derive(Deserialize)]
pub struct Checker {
    pub path: PathBuf,
}

/// Tests general info
#[derive(Deserialize)]
pub struct Tests {
    pub path: PathBuf,
}

/// Test group
#[derive(Deserialize)]
pub struct TestGroup {
    pub r#type: TestGroupType,
    pub tests: Vec<u8>,
    pub score: u8,
}

/// Test group's type
#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TestGroupType {
    Sample,
    Main,
}
