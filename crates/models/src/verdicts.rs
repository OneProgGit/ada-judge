use std::fmt;

use serde::{Deserialize, Serialize};

/// Subgroup's verdict
#[derive(Clone, PartialEq, Eq, sqlx::Type, Debug, Serialize, Deserialize)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
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
            SubgroupVerdict::Ok => "OK",
            SubgroupVerdict::RuntimeError => "RE",
            SubgroupVerdict::TimeLimitExceeded => "TLE",
            SubgroupVerdict::MemoryLimitExceeded => "MLE",
            SubgroupVerdict::SecurityError => "SE",
            SubgroupVerdict::WrongAnswer => "WA",
            SubgroupVerdict::PresentationError => "PE",
            SubgroupVerdict::Skipped => "SK",
            SubgroupVerdict::Testing => "TEST",
        };
        write!(f, "{}", converted)
    }
}

impl std::error::Error for SubgroupVerdict {}

/// Total testing verdict
#[derive(Clone, PartialEq, Eq, sqlx::Type, Debug, Serialize, Deserialize)]
#[sqlx(type_name = "text", rename_all = "snake_case")]
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
            TotalVerdict::Ok => "OK",
            TotalVerdict::PartialSolution => "PS",
            TotalVerdict::Pending => "PENDING",
            TotalVerdict::Compiling => "COMPILING",
            TotalVerdict::CompilationError => "CE",
            TotalVerdict::Testing => "TEST",
            TotalVerdict::InvalidProblem => "INP",
            TotalVerdict::InvalidRequest => "INR",
            TotalVerdict::Bug => "BUG",
        };
        write!(f, "{}", converted)
    }
}

impl std::error::Error for TotalVerdict {}
