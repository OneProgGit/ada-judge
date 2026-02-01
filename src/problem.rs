use std::time::Duration;

/// Represents a problem
struct Problem {
    title: String,
    time_limit: Duration,
    memory_limit: u64,
}

impl Default for Problem {
    fn default() -> Self {
        Self {
            title: String::new(),
            time_limit: Duration::from_secs(1),
            memory_limit: 64,
        }
    }
}

/// A builder for problems. Use it to add a title, time, memory or file size limits for a problem.
pub struct ProblemBuilder {
    problem: Problem,
}

impl From<Problem> for ProblemBuilder {
    fn from(problem: Problem) -> Self {
        Self { problem }
    }
}

impl ProblemBuilder {
    pub fn new() -> Self {
        Self {
            problem: Problem::default(),
        }
    }

    /// Update title of the problem
    pub fn with_title(mut self, title: &str) -> Self {
        self.problem.title = title.into();
        self
    }

    /// Update time limit of the problem
    pub fn with_time_limit(mut self, time_limit: Duration) -> Self {
        self.problem.time_limit = time_limit;
        self
    }

    /// Update memory limit of the problem
    pub fn with_memory_limit(mut self, memory_limit: u64) -> Self {
        self.problem.memory_limit = memory_limit;
        self
    }

    /// Save problem to database
    pub fn save(&self) {
        todo!("Implement save logic")
    }
}
