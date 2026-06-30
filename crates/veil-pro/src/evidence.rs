use crate::api::dto::RunMetaV1;
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, Instant};

pub type RunMeta = RunMetaV1;

#[derive(Clone)]
pub struct CachedRun {
    pub meta: RunMeta,
    pub report_html: String,
    pub report_json: String,
    pub effective_config: String,
    pub baseline_json: Option<String>,
    pub timestamp: Instant,
}

impl CachedRun {
    pub fn size_bytes(&self) -> usize {
        let mut size =
            self.report_html.len() + self.report_json.len() + self.effective_config.len();
        if let Some(baseline) = &self.baseline_json {
            size += baseline.len();
        }
        size
    }
}

pub struct RunCache {
    items: HashMap<String, CachedRun>,
    order: VecDeque<String>,
    max_runs_kept: usize,
    max_bytes_kept: usize,
    current_bytes: usize,
    ttl: Duration,
}

impl RunCache {
    pub fn new(max_runs_kept: usize, max_bytes_kept: usize, ttl_minutes: u64) -> Self {
        Self {
            items: HashMap::new(),
            order: VecDeque::new(),
            max_runs_kept,
            max_bytes_kept,
            current_bytes: 0,
            ttl: Duration::from_secs(ttl_minutes * 60),
        }
    }

    pub fn insert(&mut self, run_id: String, run: CachedRun) {
        self.evict_stale();

        let incoming_size = run.size_bytes();
        if incoming_size > self.max_bytes_kept {
            return;
        }

        while self.items.len() >= self.max_runs_kept
            || (self.current_bytes + incoming_size) > self.max_bytes_kept
        {
            if let Some(oldest) = self.order.pop_front() {
                if let Some(old_run) = self.items.remove(&oldest) {
                    self.current_bytes = self.current_bytes.saturating_sub(old_run.size_bytes());
                }
            } else {
                break;
            }
        }

        self.order.push_back(run_id.clone());
        self.current_bytes += incoming_size;
        self.items.insert(run_id, run);
    }

    pub fn get(&mut self, run_id: &str) -> Option<CachedRun> {
        self.evict_stale();
        self.items.get(run_id).cloned()
    }

    fn evict_stale(&mut self) {
        let now = Instant::now();
        while let Some(oldest_id) = self.order.front() {
            if let Some(run) = self.items.get(oldest_id) {
                if now.duration_since(run.timestamp) > self.ttl {
                    let id = self.order.pop_front().unwrap();
                    if let Some(old_run) = self.items.remove(&id) {
                        self.current_bytes =
                            self.current_bytes.saturating_sub(old_run.size_bytes());
                    }
                } else {
                    break;
                }
            } else {
                self.order.pop_front();
            }
        }
    }
}
