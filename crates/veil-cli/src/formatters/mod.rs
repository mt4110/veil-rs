use anyhow::Result;
use std::collections::HashMap;
use veil_core::model::{Finding, Severity};

pub mod html;
pub mod json;
pub mod markdown;
#[cfg(feature = "table")]
pub mod table;

use serde::Serialize;

#[derive(Serialize)]
pub struct Summary {
    pub total_files: usize,
    pub scanned_files: usize,
    pub skipped_files: usize,
    pub findings_count: usize,
    pub shown_findings: usize,
    // truncated is effectively limit_reached, let's keep both for backward check or prefer limit_reached
    // User requested limit_reached.
    pub limit_reached: bool,
    pub duration_ms: u128,
    pub severity_counts: HashMap<Severity, usize>,
}

impl Summary {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        total_files: usize,
        scanned_files: usize,
        skipped_files: usize,
        findings_count: usize,
        shown_findings: usize,
        limit_reached: bool,
        duration: std::time::Duration,
        severity_counts: HashMap<Severity, usize>,
    ) -> Self {
        Self {
            total_files,
            scanned_files,
            skipped_files,
            findings_count,
            shown_findings,
            limit_reached,
            duration_ms: duration.as_millis(),
            severity_counts,
        }
    }
}

pub trait Formatter {
    fn print(&self, findings: &[Finding], summary: &Summary) -> Result<()>;
}
