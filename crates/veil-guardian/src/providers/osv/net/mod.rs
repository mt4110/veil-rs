pub mod retry;
pub mod sleeper;
pub mod timeout;

pub use retry::*;
pub use sleeper::*;
pub use timeout::*;

// v0.11.5: concurrency gate (shared impl lives in guardian_next)
pub use crate::guardian_next::net::concurrency::{ConcurrencyGate, ConcurrencyPolicy};

use std::time::Duration;

#[derive(Debug, Clone)]
pub struct NetConfig {
    pub connect_timeout: Duration,
    pub per_request_timeout: Duration,
    pub total_budget: Duration,
    pub retry: retry::RetryPolicy,
    pub concurrency: ConcurrencyPolicy,
}

impl Default for NetConfig {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(2),
            per_request_timeout: Duration::from_secs(8),
            total_budget: Duration::from_secs(20),
            retry: retry::RetryPolicy::default(),
            concurrency: ConcurrencyPolicy { max_in_flight: 8 },
        }
    }
}
