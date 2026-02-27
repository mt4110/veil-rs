use anyhow::Result;
use std::collections::HashMap;
use veil_core::model::{Finding, Severity};

pub mod html;
pub mod json;
pub mod markdown;
#[cfg(feature = "table")]
pub mod table;

pub use html::HtmlFormatter;
pub use json::JsonFormatter;
pub use markdown::MarkdownFormatter;
#[cfg(feature = "table")]
pub use table::TableFormatter;

use serde::Serialize;

#[derive(Serialize)]
pub struct Summary {
    pub total_files: usize,
    pub scanned_files: usize,
    pub skipped_files: usize,
    pub total_findings: usize,
    pub new_findings: usize,
    pub baseline_suppressed: usize,
    /// Indicates whether the scan stopped early due to limit being reached.
    pub limit_reached: bool,
    /// Indicates whether the scan stopped early due to max_file_count being reached.
    pub file_limit_reached: bool,
    pub duration_ms: u128,
    pub baseline_path: Option<String>,
    pub severity_counts: HashMap<Severity, usize>,
    pub builtin_skips: Vec<String>,
}

impl Summary {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        total_files: usize,
        scanned_files: usize,
        skipped_files: usize,
        total_findings: usize,
        new_findings: usize,
        baseline_suppressed: usize,
        limit_reached: bool,
        file_limit_reached: bool,
        duration: std::time::Duration,
        baseline_path: Option<String>,
        severity_counts: HashMap<Severity, usize>,
        builtin_skips: Vec<String>,
    ) -> Self {
        Self {
            total_files,
            scanned_files,
            skipped_files,
            total_findings,
            new_findings,
            baseline_suppressed,
            limit_reached,
            file_limit_reached,
            duration_ms: duration.as_millis(),
            baseline_path,
            severity_counts,
            builtin_skips,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FindingStatus {
    New,
    Suppressed,
}

#[derive(Debug, Clone)]
pub struct DisplayFinding {
    pub inner: Finding,
    pub status: FindingStatus,
}

pub trait Formatter {
    fn print(&self, findings: &[DisplayFinding], summary: &Summary) -> Result<()>;
}
