use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(
    feature = "sqlx",
    sqlx(type_name = "verdict", rename_all = "snake_case")
)]
#[serde(rename_all = "snake_case")]
pub enum Verdict {
    Ok,
    RuntimeError,
    TimeLimitExceeded,
    MemoryLimitExceeded,
    SecurityError,
    WrongAnswer,
    PresentationError,
    Skipped,
    Testing,
    Fail,
}

#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(
    feature = "sqlx",
    sqlx(type_name = "testing_verdict", rename_all = "snake_case")
)]
#[serde(rename_all = "snake_case")]
pub enum TestingVerdict {
    Ok,
    PartialSolution,
    Pending,
    Compiling,
    CompilationError,
    Testing,
    Fail,
}
