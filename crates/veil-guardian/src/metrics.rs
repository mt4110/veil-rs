use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

#[derive(Debug, Default)]
pub struct Metrics {
    pub start_time: Option<Instant>,

    // Time (in milliseconds)
    pub time_total_ms: AtomicU64,
    pub time_parse_ms: AtomicU64,
    pub time_osv_query_ms: AtomicU64,
    pub time_osv_details_ms: AtomicU64,
    pub time_render_ms: AtomicU64,

    // Counts
    pub req_querybatch: AtomicU64,
    pub req_details: AtomicU64,
    pub cache_fresh: AtomicU64,
    pub cache_stale: AtomicU64,
    pub cache_miss: AtomicU64,
    pub coalesced_waiters: AtomicU64,
    pub retries: AtomicU64,
    pub max_concurrency: AtomicU64,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            start_time: Some(Instant::now()),
            ..Default::default()
        }
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            time_total_ms: self.time_total_ms.load(Ordering::Relaxed),
            time_parse_ms: self.time_parse_ms.load(Ordering::Relaxed),
            time_osv_query_ms: self.time_osv_query_ms.load(Ordering::Relaxed),
            time_osv_details_ms: self.time_osv_details_ms.load(Ordering::Relaxed),
            time_render_ms: self.time_render_ms.load(Ordering::Relaxed),

            req_querybatch: self.req_querybatch.load(Ordering::Relaxed),
            req_details: self.req_details.load(Ordering::Relaxed),

            cache_fresh: self.cache_fresh.load(Ordering::Relaxed),
            cache_stale: self.cache_stale.load(Ordering::Relaxed),
            cache_miss: self.cache_miss.load(Ordering::Relaxed),

            coalesced_waiters: self.coalesced_waiters.load(Ordering::Relaxed),
            retries: self.retries.load(Ordering::Relaxed),
            max_concurrency: self.max_concurrency.load(Ordering::Relaxed),
        }
    }

    pub fn observe_concurrency(&self, current: u64) {
        let mut max = self.max_concurrency.load(Ordering::Relaxed);
        while current > max {
            match self.max_concurrency.compare_exchange_weak(
                max,
                current,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => max = actual,
            }
        }
    }
}

#[derive(Debug, Serialize)]
pub struct MetricsSnapshot {
    pub time_total_ms: u64,
    pub time_parse_ms: u64,
    pub time_osv_query_ms: u64,
    pub time_osv_details_ms: u64,
    pub time_render_ms: u64,

    pub req_querybatch: u64,
    pub req_details: u64,

    pub cache_fresh: u64,
    pub cache_stale: u64,
    pub cache_miss: u64,

    pub coalesced_waiters: u64,
    pub retries: u64,
    pub max_concurrency: u64,
}

impl std::fmt::Display for Metrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let snap = self.snapshot();
        // Calculate total time safely if start_time is set
        let total = if let Some(start) = self.start_time {
            start.elapsed().as_millis() as u64
        } else {
            snap.time_total_ms
        };

        writeln!(f, "Performance Metrics:")?;
        writeln!(f, "  Time:")?;
        writeln!(f, "    Total:       {}ms", total)?;
        writeln!(f, "    Parse:       {}ms", snap.time_parse_ms)?;
        writeln!(f, "    OSV Query:   {}ms", snap.time_osv_query_ms)?;
        writeln!(f, "    OSV Details: {}ms", snap.time_osv_details_ms)?;
        writeln!(f, "    Render:      {}ms", snap.time_render_ms)?;
        writeln!(f, "  Counts:")?;
        writeln!(
            f,
            "    API Queries: {} (Batch) + {} (Details)",
            snap.req_querybatch, snap.req_details
        )?;
        writeln!(
            f,
            "    Cache:       {} fresh, {} stale, {} miss",
            snap.cache_fresh, snap.cache_stale, snap.cache_miss
        )?;
        writeln!(f, "    Coalesced:   {}", snap.coalesced_waiters)?;
        writeln!(f, "    Retries:     {}", snap.retries)?;
        writeln!(f, "    Max Concurr: {}", snap.max_concurrency)?;
        Ok(())
    }
}
