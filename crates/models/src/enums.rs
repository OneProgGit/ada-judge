use serde::{Deserialize, Serialize};
use std::fmt;

/// Verdicts
#[derive(Clone, PartialEq, Eq, sqlx::Type, Debug, Serialize, Deserialize)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum AdaJudgeVerdict {
    Ok,
    CompilationError,
    RuntimeError,
    TimeLimitExceeded,
    MemoryLimitExceeded,
    SecurityError,
    WrongAnswer,
    PresentationError,
    Skipped,
    Testing,
}

/// Statuses
#[derive(Clone, PartialEq, Eq, sqlx::Type, Debug, Serialize, Deserialize)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum AdaJudgeTotalVerdict {
    Pending,
    Testing,
    Ok,
    PartialSolution,
    InvalidProblem,
}

/// Errors
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum AdaJudgeError {
    InvalidProblem,
    CheckerFailed,
    Bug,
}

impl fmt::Display for AdaJudgeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let converted = match self {
            AdaJudgeError::InvalidProblem => "Invalid problem",
            AdaJudgeError::CheckerFailed => "Checker failed",
            AdaJudgeError::Bug => "Bug",
        };
        write!(f, "{}", converted)
    }
}

impl std::error::Error for AdaJudgeError {}
