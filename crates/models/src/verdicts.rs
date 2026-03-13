use serde::{Deserialize, Serialize};

/// Subgroup's verdicts
#[derive(Clone, PartialEq, Eq, sqlx::Type, Debug, Serialize, Deserialize)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum SubgroupVerdict {
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
    Bug,
}

/// Total testing verdicts
#[derive(Clone, PartialEq, Eq, sqlx::Type, Debug, Serialize, Deserialize)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
pub enum TotalVerdict {
    Pending,
    Testing,
    Ok,
    PartialSolution,
    InvalidProblem,
    Bug,
}
