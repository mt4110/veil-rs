use crate::metrics::reason::{MetricsV1, ReasonEventV1};
use std::collections::BTreeMap;
use std::io::BufRead;

#[derive(Debug, Default)]
pub struct AggregationStats {
    pub total_lines: u64,
    pub valid_events: u64,
    pub parse_errors: u64,
}

pub fn aggregate_events<R: BufRead>(reader: R) -> (MetricsV1, AggregationStats) {
    let mut stats = AggregationStats::default();
    let mut counts: BTreeMap<String, u64> = BTreeMap::new();
    let mut hint_counts: BTreeMap<String, u64> = BTreeMap::new();

    // Iterate over lines, tolerating failures
    for line in reader.lines() {
        stats.total_lines += 1;
        match line {
            Ok(content) => {
                if content.trim().is_empty() {
                    continue;
                }
                match serde_json::from_str::<ReasonEventV1>(&content) {
                    Ok(event) => {
                        stats.valid_events += 1;
                        // Count by ReasonCode
                        *counts
                            .entry(event.reason_code.as_str().to_string())
                            .or_insert(0) += 1;
                        // Count by HintCode
                        for hint in event.hint_codes {
                            let hint_str = serde_json::to_string(&hint)
                                .unwrap_or_default()
                                .trim_matches('"')
                                .to_string();
                            *hint_counts.entry(hint_str).or_insert(0) += 1;
                        }
                    }
                    Err(_) => {
                        stats.parse_errors += 1;
                    }
                }
            }
            Err(_) => {
                // IO error on line read
                stats.parse_errors += 1;
            }
        }
    }

    let mut m = MetricsV1::new();
    m.metrics.counts_by_reason = counts;
    m.metrics.counts_by_hint = hint_counts;

    // We could add stats to meta if desired, or return them separately
    // The requirement says strict output.

    (m, stats)
}
