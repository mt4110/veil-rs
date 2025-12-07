use crate::rules::grade::Grade;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize, Default)]
pub enum Severity {
    Low,
    #[default]
    Medium,
    High,
    Critical,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Low => write!(f, "LOW"),
            Severity::Medium => write!(f, "MEDIUM"),
            Severity::High => write!(f, "HIGH"),
            Severity::Critical => write!(f, "CRITICAL"),
        }
    }
}

impl From<&str> for Severity {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "low" => Severity::Low,
            "medium" => Severity::Medium,
            "high" => Severity::High,
            "critical" => Severity::Critical,
            _ => Severity::Medium,
        }
    }
}

#[derive(Clone)]
pub struct Rule {
    pub id: String,
    pub pattern: regex::Regex,
    pub description: String,
    pub severity: Severity,
    pub score: u32,
    pub category: String,
    pub tags: Vec<String>,
    // Optional additional validation function (e.g. check digits)
    pub validator: Option<fn(&str) -> bool>,
}

impl fmt::Debug for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Rule")
            .field("id", &self.id)
            .field("pattern", &self.pattern)
            .field("description", &self.description)
            .field("severity", &self.severity)
            .field("score", &self.score)
            .field("category", &self.category)
            .field("tags", &self.tags)
            .field(
                "validator",
                &if self.validator.is_some() {
                    "Some(fn)"
                } else {
                    "None"
                },
            )
            .finish()
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Finding {
    pub path: PathBuf,
    pub line_number: usize,
    pub line_content: String,
    pub rule_id: String,
    pub masked_line: String,
    pub severity: Severity,
    // New fields for Phase 2
    pub score: u32,
    pub grade: Grade,
}
