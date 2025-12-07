// use crate::model::{Finding, Severity};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub enum Grade {
    Safe,
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for Grade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Grade::Safe => write!(f, "SAFE"),
            Grade::Low => write!(f, "LOW"),
            Grade::Medium => write!(f, "MEDIUM"),
            Grade::High => write!(f, "HIGH"),
            Grade::Critical => write!(f, "CRITICAL"),
        }
    }
}

pub fn calculate_grade(score: u32) -> Grade {
    if score >= 90 {
        Grade::Critical
    } else if score >= 70 {
        Grade::High
    } else if score >= 40 {
        Grade::Medium
    } else if score >= 10 {
        Grade::Low
    } else {
        Grade::Safe
    }
}
