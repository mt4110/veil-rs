pub mod baseline;
pub mod finding_id;
pub mod masking;
pub mod metrics;
pub mod model;
pub mod registry;
pub mod remote;
pub mod rules;
pub mod scanner;
pub mod scoring;
pub mod summary;
pub mod validators;
pub mod verify;

pub use crate::masking::{apply_masks, apply_masks_spans, MaskSpan, DEFAULT_PLACEHOLDER};
pub use finding_id::FindingId;
pub use model::{Finding, FindingSpan, Position, Range, Rule, Severity};
pub use registry::Registry;
pub use rules::builtin::{get_all_rules, get_default_rules, try_get_all_rules};
pub use rules::grade::{calculate_grade, Grade};
pub use scanner::result::ScanResult;
pub use scanner::{
    scan_content, scan_file, scan_path, utils::scan_data, DEFAULT_MAX_FILE_COUNT,
    DEFAULT_MAX_FILE_SIZE_BYTES, RULE_ID_BINARY_FILE, RULE_ID_MAX_FILE_SIZE, RULE_ID_READ_ERROR,
};
pub use scoring::{calculate_score, grade_from_score, severity_min_score, ScoreParams};
pub use verify::{verify_evidence_pack, VerifyError, VerifyOptions, VerifyResult, VerifyStatus};

// Placeholder for future remote rule fetching
