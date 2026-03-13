use std::fmt;

use serde::{Deserialize, Serialize};

/// Errors
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub enum Error {
    InvalidProblem,
    CheckerFailed,
    Bug,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let converted = match self {
            Error::InvalidProblem => "Invalid problem",
            Error::CheckerFailed => "Checker failed",
            Error::Bug => "Bug",
        };
        write!(f, "{}", converted)
    }
}

impl std::error::Error for Error {}
