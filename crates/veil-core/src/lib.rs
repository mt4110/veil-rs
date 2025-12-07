pub mod masking;
pub mod model;
pub mod rules;
pub mod scanner;

pub use masking::mask_string;
pub use model::{Finding, Rule, Severity};
pub use rules::builtin::{get_all_rules, get_default_rules};
pub use rules::grade::{calculate_grade, Grade};
pub use rules::scoring::{calculate_base_score, calculate_context_score};
pub use scanner::{scan_content, scan_path};

// Placeholder for future remote rule fetching
pub mod remote {
    use crate::model::Rule;

    pub async fn fetch_remote_rules(server_url: &str) -> Result<Vec<Rule>, String> {
        // In real implementation: HTTP GET server_url/rules -> parse JSON -> convert to Rule
        // This requires reqwest or similar in veil-core, which might be heavy.
        // For now, we return empty.
        Err(format!("Fetching from {} not implemented yet", server_url))
    }
}
