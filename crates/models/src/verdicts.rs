use std::fmt;

use serde::{Deserialize, Serialize};

/// Subgroup's verdict
#[derive(Clone, PartialEq, Eq, sqlx::Type, Debug, Serialize, Deserialize)]
#[sqlx(type_name = "subgroup_verdict", rename_all = "snake_case")]
pub enum SubgroupVerdict {
    Ok,
    RuntimeError,
    TimeLimitExceeded,
    MemoryLimitExceeded,
    SecurityError,
    WrongAnswer,
    PresentationError,
    Skipped,
    Testing,
}

impl fmt::Display for SubgroupVerdict {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let converted = match self {
            SubgroupVerdict::Ok => "Ok",
            SubgroupVerdict::RuntimeError => "RuntimeError",
            SubgroupVerdict::TimeLimitExceeded => "TimeLimitExceeded",
            SubgroupVerdict::MemoryLimitExceeded => "MemoryLimitExceeded",
            SubgroupVerdict::SecurityError => "SecurityError",
            SubgroupVerdict::WrongAnswer => "WrongAnswer",
            SubgroupVerdict::PresentationError => "PresentationError",
            SubgroupVerdict::Skipped => "Skipped",
            SubgroupVerdict::Testing => "Testing",
        };
        write!(f, "{}", converted)
    }
}

impl std::error::Error for SubgroupVerdict {}

/// Total testing verdict
#[derive(Clone, PartialEq, Eq, sqlx::Type, Debug, Serialize, Deserialize)]
#[sqlx(type_name = "total_verdict", rename_all = "snake_case")]
pub enum TotalVerdict {
    Ok,
    PartialSolution,
    Pending,
    Compiling,
    CompilationError,
    Testing,
    InvalidProblem,
    InvalidRequest,
    Bug,
}

impl fmt::Display for TotalVerdict {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let converted = match self {
            TotalVerdict::Ok => "Ok",
            TotalVerdict::PartialSolution => "PartialSolution",
            TotalVerdict::Pending => "Pending",
            TotalVerdict::Compiling => "Compiling",
            TotalVerdict::CompilationError => "CompilationError",
            TotalVerdict::Testing => "Testing",
            TotalVerdict::InvalidProblem => "InvalidProblem",
            TotalVerdict::InvalidRequest => "InvalidRequest",
            TotalVerdict::Bug => "Bug",
        };
        write!(f, "{}", converted)
    }
}

impl std::error::Error for TotalVerdict {}
