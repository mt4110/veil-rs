use crate::models::{Advisory, DatabaseSchema};
use semver::Version;
use std::collections::HashMap;
use thiserror::Error;

static BUILTIN_ADVISORIES: &str = include_str!("advisories.json5");

#[derive(Debug, Error)]
pub enum GuardianError {
    #[error("Failed to parse advisory database: {0}")]
    ParseError(#[from] json5::Error),
    #[error("Unsupported schema version: {0} (expected 1)")]
    UnsupportedSchemaVersion(u32),
    #[error("Failed to parse Cargo.lock: {0}")]
    LockfileParseError(String),
}

pub struct GuardianDb {
    // Map crate_name -> List of advisories
    index: HashMap<String, Vec<Advisory>>,
}

impl GuardianDb {
    pub fn load_builtin() -> Result<Self, GuardianError> {
        let schema: DatabaseSchema = json5::from_str(BUILTIN_ADVISORIES)?;

        if schema.schema_version != 1 {
            return Err(GuardianError::UnsupportedSchemaVersion(
                schema.schema_version,
            ));
        }

        let mut index = HashMap::new();

        for advisory in schema.advisories {
            index
                .entry(advisory.crate_name.clone())
                .or_insert_with(Vec::new)
                .push(advisory);
        }

        Ok(Self { index })
    }

    pub fn advisories_for(&self, crate_name: &str) -> &[Advisory] {
        self.index
            .get(crate_name)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn check_vulnerabilities(&self, crate_name: &str, version: &Version) -> Vec<&Advisory> {
        let advisories = self.advisories_for(crate_name);
        advisories
            .iter()
            .filter(|advisory| advisory.vulnerable_versions.matches(version))
            .collect()
    }

    pub fn is_version_vulnerable(&self, crate_name: &str, version: &Version) -> bool {
        !self.check_vulnerabilities(crate_name, version).is_empty()
    }
}
