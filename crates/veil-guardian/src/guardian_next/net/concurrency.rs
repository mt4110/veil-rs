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
        self.sem
            .clone()
            .acquire_owned()
            .await
            .expect("semaphore closed")
    }
}
