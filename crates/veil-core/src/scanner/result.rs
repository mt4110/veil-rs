use crate::model::Finding;

#[derive(Debug, Clone, Default)]
pub struct ScanResult {
    pub findings: Vec<Finding>,
    pub total_files: usize,
    pub scanned_files: usize,
    pub skipped_files: usize,
    pub limit_reached: bool,
}
