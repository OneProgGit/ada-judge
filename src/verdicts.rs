/// Verdicts
pub enum Verdict {
    Ok,
    CompilationError,
    RuntimeError,
    TimeLimitExceeded,
    MemoryLimitExceeded,
    SecurityError,
    WrongAnswer,
    PresentationError,
    Skipped,
    InvalidProblem(String),
    Fail(String),
}
