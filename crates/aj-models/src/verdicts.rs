use std::fmt;

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

impl fmt::Display for TestingVerdict {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let converted = match self {
            Self::Ok => "Ok",
            Self::PartialSolution => "PartialSolution",
            Self::Pending => "Pending",
            Self::Compiling => "Compiling",
            Self::CompilationError => "CompilationError",
            Self::Testing => "Testing",
            Self::Fail => "Fail",
        };
        write!(f, "{converted}")
    }
}

impl std::error::Error for TestingVerdict {}
