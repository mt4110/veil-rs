use crate::models::Advisory;
use serde::Serialize;

#[derive(Debug, Serialize, Default)]
pub struct ScanResult {
    pub vulnerabilities: Vec<Vulnerability>,
    pub scanned_crates: usize,
}

#[derive(Debug, Serialize)]
pub struct Vulnerability {
    pub crate_name: String,
    pub version: String,
    pub advisories: Vec<Advisory>,
}

impl ScanResult {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_clean(&self) -> bool {
        self.vulnerabilities.is_empty()
    }
}

pub enum OutputFormat {
    Human,
    Json,
}

impl ScanResult {
    pub fn display(&self, format: OutputFormat) -> String {
        match format {
            OutputFormat::Json => serde_json::to_string_pretty(self).unwrap_or_default(),
            OutputFormat::Human => {
                if self.is_clean() {
                    return format!(
                        "No vulnerabilities found in {} packages.",
                        self.scanned_crates
                    );
                }

                let mut out = String::new();
                out.push_str(&format!(
                    "Found {} vulnerabilities in {} packages scanned:\n\n",
                    self.vulnerabilities.len(),
                    self.scanned_crates
                ));

                for vuln in &self.vulnerabilities {
                    out.push_str(&format!("- {} v{}\n", vuln.crate_name, vuln.version));
                    for advisory in &vuln.advisories {
                        out.push_str(&format!("  [{}] {}\n", advisory.id, advisory.description));
                    }
                    out.push('\n');
                }
                out
            }
        }
    }
}
