use serde::{Deserialize, Serialize};
use std::fmt;

/// Verdicts
#[derive(Clone, PartialEq, Eq, sqlx::Type, Debug, Serialize, Deserialize)]
#[sqlx(type_name = "verdict", rename_all = "lowercase")]
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
}

/// Statuses
#[derive(Clone, PartialEq, Eq, sqlx::Type, Debug, Serialize, Deserialize)]
#[sqlx(type_name = "total_verdict", rename_all = "lowercase")]
pub enum AdaJudgeTotalVerdict {
    Pending,
    Testing,
    Ok,
    PartialSolution,
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
