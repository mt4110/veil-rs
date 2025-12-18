use crate::guardian_next::error::GuardianNextError;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_attempts: usize,  // 例: 4
    pub base_delay: Duration, // 例: 200ms
    pub max_delay: Duration,  // 例: 3s
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 4,
            base_delay: Duration::from_millis(200),
            max_delay: Duration::from_secs(3),
        }
    }
}

#[derive(Clone)]
pub struct RetryRunner {
    policy: RetryPolicy,
}

impl RetryRunner {
    pub fn new(policy: RetryPolicy) -> Self {
        Self { policy }
    }

    pub fn should_retry(err: &GuardianNextError) -> bool {
        match err {
            GuardianNextError::HttpStatus { status, .. } => {
                // 429 / 5xx は retry
                *status == 429 || (*status >= 500 && *status <= 599) || *status == 408
            }
            GuardianNextError::Http(_) => true, // reqwest::Error（timeout/DNS等）を含む
            _ => false,
        }
    }

    fn backoff(&self, attempt: usize) -> Duration {
        // 指数バックオフ（jitter 最小版：attempt 依存の微小揺らぎ）
        let mut d = self.policy.base_delay * (1u32 << (attempt as u32)).min(8);
        if d > self.policy.max_delay {
            d = self.policy.max_delay;
        }
        // deterministic jitter (0..=50ms)
        let jitter_ms = (attempt as u64 * 17) % 51;
        d + Duration::from_millis(jitter_ms)
    }

    pub async fn run<F, Fut, T>(&self, mut f: F) -> Result<T, GuardianNextError>
    where
        F: FnMut(usize) -> Fut,
        Fut: std::future::Future<Output = Result<T, GuardianNextError>>,
    {
        let mut last_err: Option<GuardianNextError> = None;

        for attempt in 0..self.policy.max_attempts {
            match f(attempt).await {
                Ok(v) => return Ok(v),
                Err(e) => {
                    let retry = Self::should_retry(&e) && attempt + 1 < self.policy.max_attempts;
                    if !retry {
                        return Err(e);
                    }
                    last_err = Some(e);
                    sleep(self.backoff(attempt)).await;
                }
            }
        }

        Err(last_err.unwrap_or(GuardianNextError::Internal(
            "retry runner ended unexpectedly".into(),
        )))
    }
}
