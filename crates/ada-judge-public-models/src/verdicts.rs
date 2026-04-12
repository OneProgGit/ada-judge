//! Testing verdicts

use serde::{Deserialize, Serialize};
use std::fmt;

/// Subgroup's verdict
#[derive(Clone, Default, PartialEq, Eq, sqlx::Type, Debug, Serialize, Deserialize)]
#[sqlx(type_name = "subgroup_verdict", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum SubgroupVerdict {
    /// Ok, full score
    #[default]
    Ok,
    /// Solution panicked
    RuntimeError,
    /// Solution is too slow
    TimeLimitExceeded,
    /// Solution uses too much memory
    MemoryLimitExceeded,
    /// Solution violates security (e.g. tries to access the Internet)
    SecurityError,
    /// Solution gives the wrong answer
    WrongAnswer,
    /// Solution prints answer incorrectly
    PresentationError,
    /// Skipped testing for this subgroup because some of it's dependencies' verdicts' aren't `Ok`
    Skipped,
    /// Testing on this subgroup now
    Testing,
}

impl fmt::Display for SubgroupVerdict {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let converted = match self {
            Self::Ok => "Ok",
            Self::RuntimeError => "RuntimeError",
            Self::TimeLimitExceeded => "TimeLimitExceeded",
            Self::MemoryLimitExceeded => "MemoryLimitExceeded",
            Self::SecurityError => "SecurityError",
            Self::WrongAnswer => "WrongAnswer",
            Self::PresentationError => "PresentationError",
            Self::Skipped => "Skipped",
            Self::Testing => "Testing",
        };
        write!(f, "{converted}")
    }
}

impl std::error::Error for SubgroupVerdict {}

/// Total testing verdict
#[derive(Clone, PartialEq, Eq, sqlx::Type, Debug, Serialize, Deserialize)]
#[sqlx(type_name = "total_verdict", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TotalVerdict {
    /// Ok, full score
    Ok,
    /// Not full score (may be zero)
    PartialSolution,
    /// In testing queue now
    Pending,
    /// Compiling solution now
    Compiling,
    /// Compilation error
    CompilationError,
    /// Testing this submission now
    Testing,
    /// Invalid problem (e.g. invalid config)
    InvalidProblem,
    /// Invalid request (e.g. problem not exist)
    InvalidRequest,
    /// Internal error
    Bug,
}

impl fmt::Display for TotalVerdict {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let converted = match self {
            Self::Ok => "Ok",
            Self::PartialSolution => "PartialSolution",
            Self::Pending => "Pending",
            Self::Compiling => "Compiling",
            Self::CompilationError => "CompilationError",
            Self::Testing => "Testing",
            Self::InvalidProblem => "InvalidProblem",
            Self::InvalidRequest => "InvalidRequest",
            Self::Bug => "Bug",
        };
        write!(f, "{converted}")
    }
}

impl std::error::Error for TotalVerdict {}
