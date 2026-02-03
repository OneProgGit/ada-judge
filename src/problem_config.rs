use std::path::PathBuf;

use serde::Deserialize;

/// Problem's config
#[derive(Deserialize)]
pub struct ProblemConfig {
    general: General,
    limits: Limits,
    checker: Checker,
    test_groups: Vec<TestGroup>,
}

/// General data
#[derive(Deserialize)]
struct General {
    name: String,
}

/// Limits
#[derive(Deserialize)]
struct Limits {
    time_limit_ms: u16,
    memory_limit_mb: u16,
}

/// Checker info
#[derive(Deserialize)]
struct Checker {
    path: PathBuf,
}

/// Test group
#[derive(Deserialize)]
struct TestGroup {
    r#type: TestGroupType,
    tests: Vec<PathBuf>,
    score: u8,
}

/// Test group's type
#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum TestGroupType {
    Sample,
    Main,
}
