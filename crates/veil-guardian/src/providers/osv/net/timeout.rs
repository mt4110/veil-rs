use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct TimeBudget {
    deadline: Instant,
}

impl TimeBudget {
    pub fn new(total: Duration) -> Self {
        Self {
            deadline: Instant::now() + total,
        }
    }

    pub fn remaining(&self) -> Option<Duration> {
        self.deadline.checked_duration_since(Instant::now())
    }
}

/// Returns the shorter of `per_request` or `budget.remaining()`.
/// If no budget remains, returns None.
pub fn clamp_timeout(budget: &TimeBudget, per_request: Duration) -> Option<Duration> {
    budget.remaining().map(|rem| rem.min(per_request))
}
