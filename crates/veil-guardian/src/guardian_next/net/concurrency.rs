use std::sync::Arc;
use tokio::sync::Semaphore;

#[derive(Debug, Clone)]
pub struct ConcurrencyPolicy {
    pub max_in_flight: usize,
}

#[derive(Clone)]
pub struct ConcurrencyGate {
    sem: Arc<Semaphore>,
}

impl ConcurrencyGate {
    pub fn new(policy: ConcurrencyPolicy) -> Self {
        let n = policy.max_in_flight.max(1);
        Self {
            sem: Arc::new(Semaphore::new(n)),
        }
    }

    pub async fn acquire(&self) -> tokio::sync::OwnedSemaphorePermit {
        self.acquire_with_metrics(None).await
    }

    pub async fn acquire_with_metrics(
        &self,
        metrics: Option<&crate::Metrics>,
    ) -> tokio::sync::OwnedSemaphorePermit {
        // Optimistic check (try_acquire) could be added here for 0-wait metric
        let start = if metrics.is_some() {
            Some(std::time::Instant::now())
        } else {
            None
        };

        let permit = self
            .sem
            .clone()
            .acquire_owned()
            .await
            .expect("semaphore closed");

        if let Some(m) = metrics {
            if let Some(s) = start {
                let elapsed = s.elapsed();
                if elapsed.as_millis() > 0 {
                    m.gate_wait_count
                        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    m.gate_wait_ms.fetch_add(
                        elapsed.as_millis() as u64,
                        std::sync::atomic::Ordering::Relaxed,
                    );
                }
            }
        }
        permit
    }
}
