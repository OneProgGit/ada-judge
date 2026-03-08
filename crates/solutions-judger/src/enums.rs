use serde::{Deserialize, Serialize};

/// Verdicts
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
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
    Pending,
    TestingOnSubgroup(usize),
}

/// Errors
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum AdaJudgeError {
    InvalidProblem,
    CheckerFailed,
    Bug,
}
