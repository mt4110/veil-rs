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
    pub duration_ms: u128,
    pub severity_counts: HashMap<Severity, usize>,
}

pub trait Formatter {
    fn print(&self, findings: &[Finding], summary: &Summary) -> Result<()>;
}
