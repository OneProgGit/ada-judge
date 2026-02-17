use std::path::PathBuf;

use serde::Deserialize;

/// Problem's config
#[derive(Deserialize, Clone)]
pub(crate) struct ProblemConfig {
    pub general: General,
    pub limits: Limits,
    pub checker: Checker,
    pub tests: Tests,
    pub test_groups: Vec<TestGroup>,
}

/// General data
#[derive(Deserialize, Clone)]
pub(crate) struct General {
    pub name: String,
}

/// Limits
#[derive(Deserialize, Clone)]
pub(crate) struct Limits {
    pub time_limit_ms: u64,
    pub memory_limit_mb: u64,
}

/// Checker info
#[derive(Deserialize, Clone)]
pub(crate) struct Checker {
    pub path: PathBuf,
}

/// Tests general info
#[derive(Deserialize, Clone)]
pub(crate) struct Tests {
    pub path: PathBuf,
}

/// Test group
#[derive(Deserialize, Clone)]
pub(crate) struct TestGroup {
    pub r#type: TestGroupType,
    pub tests: Vec<u8>,
    pub score: u8,
}

/// Test group's type
#[derive(Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub(crate) enum TestGroupType {
    Sample,
    Main,
}
