use regex::Regex;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read};
use std::path::{Component, Path};
use thiserror::Error;
use zip::ZipArchive;

#[derive(Debug, Error)]
pub enum VerifyError {
    #[error("ZIP file error: {0}")]
    ZipError(#[from] zip::result::ZipError),
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),
    #[error("Missing required file: {0}")]
    MissingFile(String),
    #[error("ZipSlip security violation: Invalid path {0}")]
    ZipSlipViolation(String),
    #[error("ZipBomb security violation: {0}")]
    ZipBombViolation(String),
    #[error("Schema validation failed: {0}")]
    SchemaViolation(String),
    #[error("Hash mismatch for {0} (expected {1}, got {2})")]
    HashMismatch(String, String, String),
    #[error("Token leakage detected: {0}")]
    LeakageDetected(String),
    #[error("Invalid JSON parsing: {0}")]
    JsonParseError(#[from] serde_json::Error),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerifyStatus {
    Ok,
    PolicyViolation, // exit 1 (e.g., --require-complete failure)
    Error,           // exit 2
}

#[derive(Debug, Clone)]
pub struct VerifyResult {
    pub status: VerifyStatus,
    pub is_complete: bool,
    pub findings_count: usize,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct VerifyOptions {
    pub expect_run_meta_sha256: Option<String>,
    pub require_complete: bool,
    pub fail_on_findings: Option<usize>,
    pub max_zip_bytes: u64,
    pub max_entry_bytes: u64,
    pub max_total_bytes: u64,
    pub max_files: usize,
}

impl Default for VerifyOptions {
    fn default() -> Self {
        Self {
            expect_run_meta_sha256: None,
            require_complete: false,
            fail_on_findings: None,
            max_zip_bytes: 500 * 1024 * 1024,    // 500MB
            max_entry_bytes: 200 * 1024 * 1024,  // 200MB
            max_total_bytes: 1024 * 1024 * 1024, // 1GB
            max_files: 64,
        }
    }
}

fn sha256_hex(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

fn validate_v1_run_result(run_meta: &serde_json::Value) -> Result<(), VerifyError> {
    let result = run_meta
        .get("result")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| {
            VerifyError::SchemaViolation(
                "run_meta.json result must be an object for veil-pro-run-meta-v1".to_string(),
            )
        })?;
    let result_fields = [
        "status",
        "exitCode",
        "limitReached",
        "limitReasons",
        "summary",
    ];
    for key in result.keys() {
        if !result_fields.contains(&key.as_str()) {
            return Err(VerifyError::SchemaViolation(format!(
                "run_meta.json result contains unknown field: {key}"
            )));
        }
    }
    for field in result_fields {
        if !result.contains_key(field) {
            return Err(VerifyError::SchemaViolation(format!(
                "run_meta.json result missing required field: {field}"
            )));
        }
    }
    if !result
        .get("status")
        .is_some_and(serde_json::Value::is_string)
    {
        return Err(VerifyError::SchemaViolation(
            "run_meta.json result.status must be a string".to_string(),
        ));
    }
    if result
        .get("exitCode")
        .and_then(serde_json::Value::as_u64)
        .is_none_or(|code| code > 2)
    {
        return Err(VerifyError::SchemaViolation(
            "run_meta.json result.exitCode must be 0, 1, or 2".to_string(),
        ));
    }
    if !result
        .get("limitReached")
        .is_some_and(serde_json::Value::is_boolean)
    {
        return Err(VerifyError::SchemaViolation(
            "run_meta.json result.limitReached must be a boolean".to_string(),
        ));
    }
    if !result
        .get("limitReasons")
        .and_then(serde_json::Value::as_array)
        .is_some_and(|reasons| reasons.iter().all(serde_json::Value::is_string))
    {
        return Err(VerifyError::SchemaViolation(
            "run_meta.json result.limitReasons must be an array of strings".to_string(),
        ));
    }

    let summary = result
        .get("summary")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| {
            VerifyError::SchemaViolation(
                "run_meta.json result.summary must be an object".to_string(),
            )
        })?;
    let summary_fields = [
        "totalFindings",
        "suppressedFindings",
        "effectiveFindings",
        "severityCounts",
        "allSeverityCounts",
        "suppressedSeverityCounts",
        "coverageComplete",
    ];
    for key in summary.keys() {
        if !summary_fields.contains(&key.as_str()) {
            return Err(VerifyError::SchemaViolation(format!(
                "run_meta.json result.summary contains unknown field: {key}"
            )));
        }
    }
    for field in summary_fields {
        if !summary.contains_key(field) {
            return Err(VerifyError::SchemaViolation(format!(
                "run_meta.json result.summary missing required field: {field}"
            )));
        }
    }
    for field in ["totalFindings", "suppressedFindings", "effectiveFindings"] {
        if !summary.get(field).is_some_and(serde_json::Value::is_u64) {
            return Err(VerifyError::SchemaViolation(format!(
                "run_meta.json result.summary.{field} must be a non-negative integer"
            )));
        }
    }
    for field in [
        "severityCounts",
        "allSeverityCounts",
        "suppressedSeverityCounts",
    ] {
        if !summary.get(field).is_some_and(serde_json::Value::is_object) {
            return Err(VerifyError::SchemaViolation(format!(
                "run_meta.json result.summary.{field} must be an object"
            )));
        }
    }
    if !summary
        .get("coverageComplete")
        .is_some_and(serde_json::Value::is_boolean)
    {
        return Err(VerifyError::SchemaViolation(
            "run_meta.json result.summary.coverageComplete must be a boolean".to_string(),
        ));
    }

    Ok(())
}

pub fn verify_evidence_pack(
    zip_path: &Path,
    options: &VerifyOptions,
) -> Result<VerifyResult, VerifyError> {
    // 1. Check max_zip_bytes
    let metadata = std::fs::metadata(zip_path)?;
    if metadata.len() > options.max_zip_bytes {
        return Err(VerifyError::ZipBombViolation(format!(
            "ZIP size {} exceeds limit {}",
            metadata.len(),
            options.max_zip_bytes
        )));
    }

    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    if archive.len() > options.max_files {
        return Err(VerifyError::ZipBombViolation(format!(
            "ZIP contains {} files, exceeding limit {}",
            archive.len(),
            options.max_files
        )));
    }

    let leakage_regex =
        Regex::new(r"(?i)(#token=|\?token=|Authorization:\s*Bearer\s+[A-Za-z0-9._~-]{16,})")
            .unwrap();

    let mut total_uncompressed_bytes = 0u64;
    let mut extracted_files: HashMap<String, String> = HashMap::new(); // path -> hex_hash
    let mut extracted_content: HashMap<String, Vec<u8>> = HashMap::new();

    // 2. Stream Process ZIP (Anti-ZipSlip, Anti-ZipBomb, Leakage Check)
    for i in 0..archive.len() {
        let mut zip_entry = archive.by_index(i)?;
        let name = zip_entry.name().to_string();

        if name.ends_with('/') {
            continue;
        }

        if name == "baseline.json" {
            return Err(VerifyError::SchemaViolation(
                "baseline.json is not supported in Evidence Pack v1; use veil.baseline.json"
                    .to_string(),
            ));
        }

        // Anti-ZipSlip
        let p = Path::new(&name);
        for comp in p.components() {
            match comp {
                Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                    return Err(VerifyError::ZipSlipViolation(name.clone()));
                }
                _ => {}
            }
        }

        // Ignore Mac __MACOSX weirdness if it exists
        if name.starts_with("__MACOSX") || name.contains(".DS_Store") {
            continue;
        }

        let uncompressed_size = zip_entry.size();
        if uncompressed_size > options.max_entry_bytes {
            return Err(VerifyError::ZipBombViolation(format!(
                "File {} exceeds entry size limit {}",
                name, options.max_entry_bytes
            )));
        }

        total_uncompressed_bytes += uncompressed_size;
        if total_uncompressed_bytes > options.max_total_bytes {
            return Err(VerifyError::ZipBombViolation(format!(
                "Total expanded size exceeds {}",
                options.max_total_bytes
            )));
        }

        let mut buf = Vec::with_capacity(uncompressed_size as usize);
        zip_entry.read_to_end(&mut buf)?;

        // Stream Leakage Check
        if let Ok(text) = std::str::from_utf8(&buf) {
            if let Some(mat) = leakage_regex.find(text) {
                // Return a truncated matched context for the error to avoid printing a huge secret to the console
                let context = &text[mat.start()..std::cmp::min(mat.end() + 10, text.len())];
                return Err(VerifyError::LeakageDetected(format!(
                    "In file '{}': matched pattern around '{}...'",
                    name, context
                )));
            }
        }

        // Calculate actual hash of streamed files
        let hash = sha256_hex(&buf);
        if extracted_files.insert(name.clone(), hash).is_some() {
            return Err(VerifyError::ZipBombViolation(format!(
                "Duplicate file entry detected in ZIP: {}",
                name
            )));
        }

        // Keep required files in memory to parse them for structural validations
        if name == "run_meta.json" || name == "report.json" {
            extracted_content.insert(name, buf);
        }
    }

    // 3. Ensure Required Files
    for req in [
        "run_meta.json",
        "report.json",
        "report.html",
        "effective_config.toml",
    ] {
        if !extracted_files.contains_key(req) {
            return Err(VerifyError::MissingFile(req.to_string()));
        }
    }

    // 4. Validate run_meta.json
    let run_meta_buf = extracted_content
        .get("run_meta.json")
        .ok_or_else(|| VerifyError::MissingFile("run_meta.json".to_string()))?;

    // Check external hash anchor if provided
    if let Some(expected_meta_hash) = &options.expect_run_meta_sha256 {
        let actual_hash = extracted_files.get("run_meta.json").unwrap();
        if actual_hash != expected_meta_hash {
            return Err(VerifyError::HashMismatch(
                "run_meta.json (External Anchor)".to_string(),
                expected_meta_hash.clone(),
                actual_hash.clone(),
            ));
        }
    }

    let run_meta: serde_json::Value = serde_json::from_slice(run_meta_buf)?;

    let schema_ver = run_meta
        .get("schemaVersion")
        .or_else(|| run_meta.get("schema_version"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default();
    if schema_ver != "veil-pro-run-meta-v1" {
        return Err(VerifyError::SchemaViolation(format!(
            "Unsupported run_meta.json schema: {}",
            schema_ver
        )));
    }
    validate_v1_run_result(&run_meta)?;

    let limit_reached = run_meta
        .pointer("/result/limitReached")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let is_complete = run_meta
        .pointer("/result/summary/coverageComplete")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(!limit_reached)
        && !limit_reached;

    let findings_count = run_meta
        .pointer("/result/summary/effectiveFindings")
        .and_then(serde_json::Value::as_u64)
        .map(|count| count as usize)
        .unwrap_or(0);

    // 5. Match hashes against run_meta.json tracking
    if let Some(artifacts_map) = run_meta
        .get("artifacts")
        .and_then(serde_json::Value::as_object)
    {
        let expected_files = [
            ("reportHtml", "report_html", false),
            ("reportJson", "report_json", false),
            ("effectiveConfig", "effective_config", false),
            ("baseline", "baseline", true),
        ];
        for (camel_key, snake_key, optional) in expected_files {
            let Some(art) = artifacts_map
                .get(camel_key)
                .or_else(|| artifacts_map.get(snake_key))
            else {
                if optional {
                    continue;
                }
                return Err(VerifyError::SchemaViolation(format!(
                    "artifact {} missing from run_meta.json",
                    camel_key
                )));
            };

            let expected_path = art
                .get("path")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| {
                    VerifyError::SchemaViolation(format!(
                        "artifact {} path missing from run_meta.json",
                        camel_key
                    ))
                })?;
            if camel_key == "baseline" && expected_path != crate::baseline::DEFAULT_BASELINE_FILE {
                return Err(VerifyError::SchemaViolation(format!(
                    "artifact baseline path must be {}",
                    crate::baseline::DEFAULT_BASELINE_FILE
                )));
            }
            let expected_hash = art
                .get("sha256")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| {
                    VerifyError::SchemaViolation(format!(
                        "artifact {} sha256 missing from run_meta.json",
                        camel_key
                    ))
                })?;

            if let Some(actual_hash) = extracted_files.get(expected_path) {
                if actual_hash != expected_hash {
                    return Err(VerifyError::HashMismatch(
                        expected_path.to_string(),
                        expected_hash.to_string(),
                        actual_hash.clone(),
                    ));
                }
            } else {
                return Err(VerifyError::MissingFile(expected_path.to_string()));
            }
        }
    } else {
        return Err(VerifyError::SchemaViolation(
            "artifacts map missing from run_meta.json".to_string(),
        ));
    }

    // 6. Validate Application Policies
    if options.require_complete && !is_complete {
        return Ok(VerifyResult {
            status: VerifyStatus::PolicyViolation,
            is_complete,
            findings_count,
            message: "Policy Violation: Evidence pack is incomplete (limit reached during scan)."
                .to_string(),
        });
    }

    if let Some(threshold) = options.fail_on_findings {
        if findings_count >= threshold {
            return Ok(VerifyResult {
                status: VerifyStatus::PolicyViolation,
                is_complete,
                findings_count,
                message: format!("Policy Violation: Extracted {} findings, meeting or exceeding the configured threshold ({}).", findings_count, threshold),
            });
        }
    }

    Ok(VerifyResult {
        status: VerifyStatus::Ok,
        is_complete,
        findings_count,
        message: "✅ Evidence Pack Validation Passed".to_string(),
    })
}
