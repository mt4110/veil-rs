pub mod masking;
pub mod model;
pub mod remote;
pub mod rules;
pub mod scanner;
pub mod scoring;

pub use crate::masking::{apply_masks, DEFAULT_PLACEHOLDER};
pub use model::{Finding, Rule, Severity};
pub use rules::builtin::{get_all_rules, get_default_rules};
pub use rules::grade::{calculate_grade, Grade};
pub use scanner::result::ScanResult;
pub use scanner::{
    scan_content, scan_file, scan_path, utils::scan_data, RULE_ID_BINARY_FILE,
    RULE_ID_MAX_FILE_SIZE,
};
pub use scoring::{calculate_score, grade_from_score, ScoreParams};

// Placeholder for future remote rule fetching
