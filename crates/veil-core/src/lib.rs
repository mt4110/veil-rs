pub mod masking;
pub mod model;
pub mod remote;
pub mod rules;
pub mod scanner;

pub use masking::{mask_ranges, mask_string};
pub use model::{Finding, Rule, Severity};
pub use rules::builtin::{get_all_rules, get_default_rules};
pub use rules::grade::{calculate_grade, Grade};
pub use rules::scoring::{calculate_base_score, calculate_context_score};
pub use scanner::{scan_content, scan_file, scan_path};

// Placeholder for future remote rule fetching
