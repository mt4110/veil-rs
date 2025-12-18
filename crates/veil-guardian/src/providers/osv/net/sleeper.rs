use async_trait::async_trait;
use std::sync::Mutex;
use std::time::Duration;

#[async_trait]
pub trait Sleeper: Send + Sync + 'static {
    async fn sleep(&self, duration: Duration);
}

pub struct TokioSleeper;

#[async_trait]
impl Sleeper for TokioSleeper {
    async fn sleep(&self, duration: Duration) {
        tokio::time::sleep(duration).await;
    }
}

/// For testing: matches user specification accurately
#[derive(Default)]
pub struct RecordingSleeper {
    sleeps: Mutex<Vec<Duration>>,
}

impl RecordingSleeper {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn sleeps(&self) -> Vec<Duration> {
        self.sleeps.lock().unwrap().clone()
    }
}

#[async_trait]
impl Sleeper for RecordingSleeper {
    async fn sleep(&self, duration: Duration) {
        self.sleeps.lock().unwrap().push(duration);
    }
}
