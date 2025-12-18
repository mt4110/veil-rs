use reqwest::header::HeaderMap;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_attempts: usize,
    pub base_delay: Duration,
    pub max_delay: Duration,
    pub jitter_factor: f64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(200),
            max_delay: Duration::from_secs(2),
            jitter_factor: 0.2,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryClass {
    Success,
    RetryAfter(Duration),
    Backoff,
    Fatal,
}

pub fn parse_retry_after(headers: &HeaderMap) -> Option<Duration> {
    let v = headers.get("retry-after")?.to_str().ok()?;
    let secs: u64 = v.parse().ok()?;
    Some(Duration::from_secs(secs))
}

pub fn classify_response(status: u16, headers: &HeaderMap) -> RetryClass {
    if (200..300).contains(&status) || status == 304 {
        return RetryClass::Success;
    }

    if status == 429 || status == 503 {
        if let Some(d) = parse_retry_after(headers) {
            return RetryClass::RetryAfter(d);
        }
        return RetryClass::Backoff;
    }

    if status == 408 || (500..600).contains(&status) {
        return RetryClass::Backoff;
    }

    RetryClass::Fatal
}

pub fn classify_error(e: &reqwest::Error) -> RetryClass {
    if e.is_timeout() || e.is_connect() {
        return RetryClass::Backoff;
    }
    if e.is_builder() {
        return RetryClass::Fatal;
    }
    RetryClass::Backoff
}

// No changes needed for pure logic.
pub fn backoff_delay(policy: &RetryPolicy, attempt: usize) -> Duration {
    let pow = 1u32
        .checked_shl((attempt.saturating_sub(1)) as u32)
        .unwrap_or(u32::MAX);
    let raw = policy.base_delay.saturating_mul(pow);
    let capped = raw.min(policy.max_delay);
    apply_jitter(capped, policy.jitter_factor)
}

fn apply_jitter(delay: Duration, jitter_factor: f64) -> Duration {
    if jitter_factor <= 0.0 {
        return delay;
    }

    let nanos = delay.as_nanos() as f64;
    let span = nanos * jitter_factor; // ±span
    let r = pseudo_rand_0_1();
    let offset = (r * 2.0 - 1.0) * span;
    let out = (nanos + offset).max(0.0);
    Duration::from_nanos(out as u64)
}

fn pseudo_rand_0_1() -> f64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let t = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        .to_le_bytes();

    let h = blake3::hash(&t);
    let mut b = [0u8; 8];
    b.copy_from_slice(&h.as_bytes()[..8]);
    let u = u64::from_le_bytes(b);
    (u as f64) / (u64::MAX as f64)
}
