/// Verdicts
pub enum Verdict {
    Ok,
    Rejected,
    CompilationError,
    RuntimeError,
    TimeLimitExceeded,
    MemoryLimitExceeded,
    SecurityError,
    WrongAnswer,
    PresentationError,
    Skipped,
    InvalidProblem,
}
