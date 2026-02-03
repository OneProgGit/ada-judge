//! Judgement system made with Rust.

use std::{fs::read_to_string, path::PathBuf};

use crate::verdicts::Verdict;

pub mod problem_config;
pub mod verdicts;

/// Test solution and return a verdict for each subgroup.
pub fn test(problem: PathBuf, submission: PathBuf) -> Result<Vec<(Verdict, u16)>, Verdict> {
    let problem_cfg = read_to_string(problem).map_err(|_| Verdict::InvalidProblem)?;

    Ok(Vec::new())
}
