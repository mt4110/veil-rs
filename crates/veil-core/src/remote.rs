use crate::model::{Rule, Severity};
use regex::Regex;
use serde::Deserialize;
use std::time::Duration;

#[derive(Debug, Deserialize)]
struct RemoteRule {
    pub id: String,
    pub pattern: String,
    pub description: String,
    pub severity: String,
    pub score: u32,
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
}

#[derive(thiserror::Error, Debug)]
pub enum RemoteError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Invalid regex in remote rule {id}: {error}")]
    RegexError { id: String, error: regex::Error },
    #[error("Protocol error: {0}")]
    Protocol(String),
}

pub fn fetch_remote_rules(url: &str, timeout_secs: u64) -> Result<Vec<Rule>, RemoteError> {
    if !url.starts_with("https://") {
        return Err(RemoteError::Protocol("URL must use HTTPS".to_string()));
    }

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .build()?;

    let response = client.get(url).send()?;
    let remote_rules: Vec<RemoteRule> = response.json()?;

    let mut rules = Vec::new();

    for rr in remote_rules {
        let pattern = Regex::new(&rr.pattern).map_err(|e| RemoteError::RegexError {
            id: rr.id.clone(),
            error: e,
        })?;

        rules.push(Rule {
            id: rr.id,
            pattern,
            description: rr.description,
            severity: Severity::from(rr.severity.as_str()),
            score: rr.score,
            category: rr.category.unwrap_or_else(|| "remote".to_string()),
            tags: rr.tags.unwrap_or_default(),
            validator: None, // Remote rules cannot have code validators
        });
    }

    Ok(rules)
}
