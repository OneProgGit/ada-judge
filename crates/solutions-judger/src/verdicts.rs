/// Verdicts
#[derive(Clone, PartialEq, Debug)]
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
}
