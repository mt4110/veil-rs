use regex::Regex;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::collections::{HashMap, HashSet};
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

fn validate_object_schema<'a>(
    value: &'a Value,
    context: &str,
    required: &[&str],
    optional: &[&str],
) -> Result<&'a Map<String, Value>, VerifyError> {
    let object = value
        .as_object()
        .ok_or_else(|| VerifyError::SchemaViolation(format!("{context} must be an object")))?;
    for key in object.keys() {
        if !required.contains(&key.as_str()) && !optional.contains(&key.as_str()) {
            return Err(VerifyError::SchemaViolation(format!(
                "{context} contains unknown field: {key}"
            )));
        }
    }
    for field in required {
        if !object.contains_key(*field) {
            return Err(VerifyError::SchemaViolation(format!(
                "{context} missing required field: {field}"
            )));
        }
    }
    Ok(object)
}

fn validate_string_field(
    object: &Map<String, Value>,
    field: &str,
    context: &str,
) -> Result<(), VerifyError> {
    if !object.get(field).is_some_and(Value::is_string) {
        return Err(VerifyError::SchemaViolation(format!(
            "{context}.{field} must be a string"
        )));
    }
    Ok(())
}

fn validate_string_enum(
    object: &Map<String, Value>,
    field: &str,
    allowed: &[&str],
    context: &str,
) -> Result<(), VerifyError> {
    if object
        .get(field)
        .and_then(Value::as_str)
        .is_none_or(|value| !allowed.contains(&value))
    {
        return Err(VerifyError::SchemaViolation(format!(
            "{context}.{field} must be one of {}",
            allowed.join(", ")
        )));
    }
    Ok(())
}

fn validate_nullable_string_field(
    object: &Map<String, Value>,
    field: &str,
    context: &str,
) -> Result<(), VerifyError> {
    if let Some(value) = object.get(field) {
        if !value.is_null() && !value.is_string() {
            return Err(VerifyError::SchemaViolation(format!(
                "{context}.{field} must be a string or null"
            )));
        }
    }
    Ok(())
}

fn validate_nullable_string_enum(
    object: &Map<String, Value>,
    field: &str,
    allowed: &[&str],
    context: &str,
) -> Result<(), VerifyError> {
    if let Some(value) = object.get(field) {
        if value.is_null() {
            return Ok(());
        }
        if value.as_str().is_none_or(|value| !allowed.contains(&value)) {
            return Err(VerifyError::SchemaViolation(format!(
                "{context}.{field} must be one of {} or null",
                allowed.join(", ")
            )));
        }
    }
    Ok(())
}

fn validate_u64_field(
    object: &Map<String, Value>,
    field: &str,
    context: &str,
) -> Result<(), VerifyError> {
    if !object.get(field).is_some_and(Value::is_u64) {
        return Err(VerifyError::SchemaViolation(format!(
            "{context}.{field} must be a non-negative integer"
        )));
    }
    Ok(())
}

fn validate_bool_field(
    object: &Map<String, Value>,
    field: &str,
    context: &str,
) -> Result<(), VerifyError> {
    if !object.get(field).is_some_and(Value::is_boolean) {
        return Err(VerifyError::SchemaViolation(format!(
            "{context}.{field} must be a boolean"
        )));
    }
    Ok(())
}

fn validate_severity_counts(value: &Value, context: &str) -> Result<(), VerifyError> {
    let counts =
        validate_object_schema(value, context, &["Low", "Medium", "High", "Critical"], &[])?;
    for field in ["Low", "Medium", "High", "Critical"] {
        validate_u64_field(counts, field, context)?;
    }
    Ok(())
}

fn validate_evidence_summary(value: &Value, context: &str) -> Result<(), VerifyError> {
    let summary = validate_object_schema(
        value,
        context,
        &[
            "totalFindings",
            "suppressedFindings",
            "effectiveFindings",
            "severityCounts",
            "allSeverityCounts",
            "suppressedSeverityCounts",
            "coverageComplete",
        ],
        &[],
    )?;
    for field in ["totalFindings", "suppressedFindings", "effectiveFindings"] {
        validate_u64_field(summary, field, context)?;
    }
    validate_severity_counts(
        summary.get("severityCounts").unwrap(),
        &format!("{context}.severityCounts"),
    )?;
    validate_severity_counts(
        summary.get("allSeverityCounts").unwrap(),
        &format!("{context}.allSeverityCounts"),
    )?;
    validate_severity_counts(
        summary.get("suppressedSeverityCounts").unwrap(),
        &format!("{context}.suppressedSeverityCounts"),
    )?;
    validate_bool_field(summary, "coverageComplete", context)?;
    Ok(())
}

fn validate_v1_run_result(run_meta: &Value) -> Result<(), VerifyError> {
    let result = run_meta
        .get("result")
        .and_then(Value::as_object)
        .ok_or_else(|| {
            VerifyError::SchemaViolation(
                "run_meta.json result must be an object for veil-pro-run-meta-v1".to_string(),
            )
        })?;
    validate_object_schema(
        run_meta.get("result").unwrap(),
        "run_meta.json result",
        &[
            "status",
            "exitCode",
            "limitReached",
            "limitReasons",
            "summary",
        ],
        &[],
    )?;
    validate_string_enum(
        result,
        "status",
        &["success", "violation", "incomplete", "error"],
        "run_meta.json result",
    )?;
    let status = result.get("status").and_then(Value::as_str).unwrap();
    let exit_code = result
        .get("exitCode")
        .and_then(Value::as_u64)
        .ok_or_else(|| {
            VerifyError::SchemaViolation(
                "run_meta.json result.exitCode must be 0, 1, or 2".to_string(),
            )
        })?;
    if exit_code > 2 {
        return Err(VerifyError::SchemaViolation(
            "run_meta.json result.exitCode must be 0, 1, or 2".to_string(),
        ));
    }
    let expected_exit_code = match status {
        "success" => 0,
        "violation" => 1,
        "incomplete" | "error" => 2,
        _ => unreachable!("status enum validation already ran"),
    };
    if exit_code != expected_exit_code {
        return Err(VerifyError::SchemaViolation(format!(
            "run_meta.json result.exitCode must be {expected_exit_code} when status is {status}"
        )));
    }
    validate_bool_field(result, "limitReached", "run_meta.json result")?;
    if !result
        .get("limitReasons")
        .and_then(Value::as_array)
        .is_some_and(|reasons| reasons.iter().all(Value::is_string))
    {
        return Err(VerifyError::SchemaViolation(
            "run_meta.json result.limitReasons must be an array of strings".to_string(),
        ));
    }
    validate_evidence_summary(
        result.get("summary").unwrap(),
        "run_meta.json result.summary",
    )?;

    Ok(())
}

fn validate_artifact_meta(
    value: &Value,
    context: &str,
    camel_key: &str,
) -> Result<(), VerifyError> {
    let canonical_path = canonical_artifact_path(camel_key)
        .ok_or_else(|| VerifyError::SchemaViolation(format!("unknown artifact key {camel_key}")))?;
    let artifact = validate_object_schema(value, context, &["path", "sha256"], &["sizeBytes"])?;
    validate_string_field(artifact, "path", context)?;
    validate_string_field(artifact, "sha256", context)?;
    if let Some(size_bytes) = artifact.get("sizeBytes") {
        if !size_bytes.is_null() && !size_bytes.is_u64() {
            return Err(VerifyError::SchemaViolation(format!(
                "{context}.sizeBytes must be a non-negative integer or null"
            )));
        }
    }
    if artifact
        .get("path")
        .and_then(Value::as_str)
        .is_none_or(|path| path != canonical_path)
    {
        return Err(VerifyError::SchemaViolation(format!(
            "{context}.path must be {canonical_path}"
        )));
    }
    Ok(())
}

fn canonical_artifact_path(camel_key: &str) -> Option<&'static str> {
    match camel_key {
        "reportHtml" => Some("report.html"),
        "reportJson" => Some("report.json"),
        "effectiveConfig" => Some("effective_config.toml"),
        "baseline" => Some(crate::baseline::DEFAULT_BASELINE_FILE),
        _ => None,
    }
}

fn is_allowed_evidence_pack_v1_file(name: &str) -> bool {
    matches!(
        name,
        "run_meta.json" | "report.json" | "report.html" | "effective_config.toml"
    ) || name == crate::baseline::DEFAULT_BASELINE_FILE
}

fn require_canonical_artifact_path(
    artifact: &Map<String, Value>,
    camel_key: &str,
) -> Result<&'static str, VerifyError> {
    let canonical_path = canonical_artifact_path(camel_key)
        .ok_or_else(|| VerifyError::SchemaViolation(format!("unknown artifact key {camel_key}")))?;
    if artifact
        .get("path")
        .and_then(Value::as_str)
        .is_none_or(|path| path != canonical_path)
    {
        return Err(VerifyError::SchemaViolation(format!(
            "artifact {camel_key} path must be {canonical_path}"
        )));
    }
    Ok(canonical_path)
}

fn validate_evidence_artifacts(value: &Value) -> Result<(), VerifyError> {
    let artifacts = validate_object_schema(
        value,
        "run_meta.json artifacts",
        &["reportHtml", "reportJson", "effectiveConfig"],
        &["baseline"],
    )?;
    validate_artifact_meta(
        artifacts.get("reportHtml").unwrap(),
        "run_meta.json artifacts.reportHtml",
        "reportHtml",
    )?;
    validate_artifact_meta(
        artifacts.get("reportJson").unwrap(),
        "run_meta.json artifacts.reportJson",
        "reportJson",
    )?;
    validate_artifact_meta(
        artifacts.get("effectiveConfig").unwrap(),
        "run_meta.json artifacts.effectiveConfig",
        "effectiveConfig",
    )?;
    if let Some(baseline) = artifacts.get("baseline").filter(|value| !value.is_null()) {
        validate_artifact_meta(baseline, "run_meta.json artifacts.baseline", "baseline")?;
    }
    Ok(())
}

fn validate_product(value: &Value) -> Result<(), VerifyError> {
    let product = validate_object_schema(
        value,
        "run_meta.json product",
        &["name", "version"],
        &["commit", "buildProfile"],
    )?;
    validate_string_enum(
        product,
        "name",
        &["veil-pro", "veil"],
        "run_meta.json product",
    )?;
    validate_string_field(product, "version", "run_meta.json product")?;
    if product.contains_key("commit") {
        validate_nullable_string_field(product, "commit", "run_meta.json product")?;
    }
    if product.contains_key("buildProfile") {
        validate_nullable_string_enum(
            product,
            "buildProfile",
            &["debug", "release"],
            "run_meta.json product",
        )?;
    }
    Ok(())
}

fn validate_engine(value: &Value) -> Result<(), VerifyError> {
    let engine = validate_object_schema(
        value,
        "run_meta.json engine",
        &["name", "schemaVersion", "rulePacks"],
        &[],
    )?;
    validate_string_enum(engine, "name", &["veil"], "run_meta.json engine")?;
    validate_string_enum(
        engine,
        "schemaVersion",
        &["veil-v1"],
        "run_meta.json engine",
    )?;
    let rule_packs = engine
        .get("rulePacks")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            VerifyError::SchemaViolation(
                "run_meta.json engine.rulePacks must be an array".to_string(),
            )
        })?;
    for (index, rule_pack) in rule_packs.iter().enumerate() {
        let context = format!("run_meta.json engine.rulePacks[{index}]");
        let rule_pack = validate_object_schema(
            rule_pack,
            &context,
            &["name", "source"],
            &["contentSha256", "version"],
        )?;
        validate_string_field(rule_pack, "name", &context)?;
        validate_string_enum(
            rule_pack,
            "source",
            &["embedded", "local", "remote"],
            &context,
        )?;
        if rule_pack.contains_key("contentSha256") {
            validate_nullable_string_field(rule_pack, "contentSha256", &context)?;
        }
        if rule_pack.contains_key("version") {
            validate_nullable_string_field(rule_pack, "version", &context)?;
        }
    }
    Ok(())
}

fn validate_privacy(value: &Value) -> Result<(), VerifyError> {
    let privacy = validate_object_schema(
        value,
        "run_meta.json privacy",
        &["telemetry", "networkMode", "bind"],
        &[],
    )?;
    validate_string_enum(privacy, "telemetry", &["none"], "run_meta.json privacy")?;
    validate_string_enum(
        privacy,
        "networkMode",
        &["local-only", "enterprise-opt-in"],
        "run_meta.json privacy",
    )?;
    validate_string_enum(privacy, "bind", &["127.0.0.1"], "run_meta.json privacy")?;
    Ok(())
}

fn validate_v1_run_meta(run_meta: &Value) -> Result<(), VerifyError> {
    let root = validate_object_schema(
        run_meta,
        "run_meta.json",
        &[
            "schemaVersion",
            "runId",
            "generatedAtUtc",
            "product",
            "engine",
            "result",
            "artifacts",
            "privacy",
        ],
        &["extensions"],
    )?;
    validate_string_enum(
        root,
        "schemaVersion",
        &["veil-pro-run-meta-v1"],
        "run_meta.json",
    )?;
    validate_string_field(root, "runId", "run_meta.json")?;
    validate_string_field(root, "generatedAtUtc", "run_meta.json")?;
    validate_product(root.get("product").unwrap())?;
    validate_engine(root.get("engine").unwrap())?;
    validate_v1_run_result(run_meta)?;
    validate_evidence_artifacts(root.get("artifacts").unwrap())?;
    validate_privacy(root.get("privacy").unwrap())?;
    if let Some(extensions) = root.get("extensions") {
        if !extensions.is_object() && !extensions.is_null() {
            return Err(VerifyError::SchemaViolation(
                "run_meta.json extensions must be an object or null".to_string(),
            ));
        }
    }
    Ok(())
}

fn validate_safe_finding(value: &Value, context: &str) -> Result<(), VerifyError> {
    let finding = validate_object_schema(
        value,
        context,
        &[
            "findingId",
            "baselineFingerprint",
            "path",
            "lineNumber",
            "ruleId",
            "severity",
            "score",
            "grade",
            "maskedSnippet",
            "category",
            "tags",
            "baselineStatus",
        ],
        &[],
    )?;
    for field in [
        "findingId",
        "baselineFingerprint",
        "path",
        "ruleId",
        "maskedSnippet",
        "category",
    ] {
        validate_string_field(finding, field, context)?;
    }
    if finding
        .get("lineNumber")
        .and_then(Value::as_u64)
        .is_none_or(|line| line == 0)
    {
        return Err(VerifyError::SchemaViolation(format!(
            "{context}.lineNumber must be a positive integer"
        )));
    }
    if finding
        .get("score")
        .and_then(Value::as_u64)
        .is_none_or(|score| score > 100)
    {
        return Err(VerifyError::SchemaViolation(format!(
            "{context}.score must be between 0 and 100"
        )));
    }
    validate_string_enum(
        finding,
        "severity",
        &["Low", "Medium", "High", "Critical"],
        context,
    )?;
    validate_string_enum(
        finding,
        "grade",
        &["Low", "Medium", "High", "Critical"],
        context,
    )?;
    validate_string_enum(
        finding,
        "baselineStatus",
        &["none", "new", "suppressed"],
        context,
    )?;
    let tags = finding
        .get("tags")
        .and_then(Value::as_array)
        .ok_or_else(|| VerifyError::SchemaViolation(format!("{context}.tags must be an array")))?;
    if !tags.iter().all(Value::is_string) {
        return Err(VerifyError::SchemaViolation(format!(
            "{context}.tags must contain only strings"
        )));
    }
    Ok(())
}

fn validate_evidence_report(report: &Value) -> Result<(), VerifyError> {
    let root = validate_object_schema(
        report,
        "report.json",
        &[
            "schemaVersion",
            "runId",
            "generatedAtUtc",
            "summary",
            "findings",
        ],
        &[],
    )?;
    validate_string_enum(
        root,
        "schemaVersion",
        &["veil-evidence-report-v1"],
        "report.json",
    )?;
    validate_string_field(root, "runId", "report.json")?;
    validate_string_field(root, "generatedAtUtc", "report.json")?;
    validate_evidence_summary(root.get("summary").unwrap(), "report.json summary")?;
    let findings = root
        .get("findings")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            VerifyError::SchemaViolation("report.json findings must be an array".to_string())
        })?;
    for (index, finding) in findings.iter().enumerate() {
        validate_safe_finding(finding, &format!("report.json findings[{index}]"))?;
    }
    Ok(())
}

fn new_severity_count_map() -> HashMap<&'static str, u64> {
    HashMap::from([("Low", 0), ("Medium", 0), ("High", 0), ("Critical", 0)])
}

fn increment_severity_count(
    counts: &mut HashMap<&'static str, u64>,
    severity: &str,
    context: &str,
) -> Result<(), VerifyError> {
    let severity = match severity {
        "Low" => "Low",
        "Medium" => "Medium",
        "High" => "High",
        "Critical" => "Critical",
        _ => {
            return Err(VerifyError::SchemaViolation(format!(
                "{context}.severity must be one of Low, Medium, High, Critical"
            )));
        }
    };
    *counts.get_mut(severity).unwrap() += 1;
    Ok(())
}

fn compare_summary_u64(
    summary: &Value,
    field: &str,
    expected: u64,
    context: &str,
) -> Result<(), VerifyError> {
    let actual = summary.get(field).and_then(Value::as_u64).ok_or_else(|| {
        VerifyError::SchemaViolation(format!("{context}.{field} must be a non-negative integer"))
    })?;
    if actual != expected {
        return Err(VerifyError::SchemaViolation(format!(
            "{context}.{field} does not match findings baselineStatus"
        )));
    }
    Ok(())
}

fn compare_summary_severity_counts(
    summary: &Value,
    field: &str,
    expected: &HashMap<&'static str, u64>,
    context: &str,
) -> Result<(), VerifyError> {
    let counts = summary
        .get(field)
        .and_then(Value::as_object)
        .ok_or_else(|| {
            VerifyError::SchemaViolation(format!("{context}.{field} must be an object"))
        })?;
    for severity in ["Low", "Medium", "High", "Critical"] {
        let actual = counts
            .get(severity)
            .and_then(Value::as_u64)
            .ok_or_else(|| {
                VerifyError::SchemaViolation(format!(
                    "{context}.{field}.{severity} must be a non-negative integer"
                ))
            })?;
        if actual != expected[severity] {
            return Err(VerifyError::SchemaViolation(format!(
                "{context}.{field}.{severity} does not match findings baselineStatus"
            )));
        }
    }
    Ok(())
}

fn validate_report_summary_matches_findings(report: &Value) -> Result<(), VerifyError> {
    let summary = report
        .get("summary")
        .ok_or_else(|| VerifyError::SchemaViolation("report.json summary missing".to_string()))?;
    let findings = report
        .get("findings")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            VerifyError::SchemaViolation("report.json findings must be an array".to_string())
        })?;

    let mut effective_counts = new_severity_count_map();
    let mut all_counts = new_severity_count_map();
    let mut suppressed_counts = new_severity_count_map();
    let mut suppressed_findings = 0u64;

    for (index, finding) in findings.iter().enumerate() {
        let context = format!("report.json findings[{index}]");
        let severity = finding
            .get("severity")
            .and_then(Value::as_str)
            .ok_or_else(|| VerifyError::SchemaViolation(format!("{context}.severity missing")))?;
        let baseline_status = finding
            .get("baselineStatus")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                VerifyError::SchemaViolation(format!("{context}.baselineStatus missing"))
            })?;

        increment_severity_count(&mut all_counts, severity, &context)?;
        if baseline_status == "suppressed" {
            suppressed_findings += 1;
            increment_severity_count(&mut suppressed_counts, severity, &context)?;
        } else {
            increment_severity_count(&mut effective_counts, severity, &context)?;
        }
    }

    let total_findings = findings.len() as u64;
    let effective_findings = total_findings - suppressed_findings;
    compare_summary_u64(
        summary,
        "totalFindings",
        total_findings,
        "report.json summary",
    )?;
    compare_summary_u64(
        summary,
        "suppressedFindings",
        suppressed_findings,
        "report.json summary",
    )?;
    compare_summary_u64(
        summary,
        "effectiveFindings",
        effective_findings,
        "report.json summary",
    )?;
    compare_summary_severity_counts(
        summary,
        "severityCounts",
        &effective_counts,
        "report.json summary",
    )?;
    compare_summary_severity_counts(
        summary,
        "allSeverityCounts",
        &all_counts,
        "report.json summary",
    )?;
    compare_summary_severity_counts(
        summary,
        "suppressedSeverityCounts",
        &suppressed_counts,
        "report.json summary",
    )?;
    Ok(())
}

fn parse_baseline_fingerprints(content: &[u8]) -> Result<HashSet<String>, VerifyError> {
    let baseline: Value = serde_json::from_slice(content)?;
    let root = validate_object_schema(
        &baseline,
        "veil.baseline.json",
        &["schema", "entries"],
        &["generated_at", "tool"],
    )?;
    validate_string_enum(
        root,
        "schema",
        &[crate::baseline::BASELINE_SCHEMA_V1],
        "veil.baseline.json",
    )?;
    let entries = root
        .get("entries")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            VerifyError::SchemaViolation("veil.baseline.json entries must be an array".to_string())
        })?;

    let mut fingerprints = HashSet::new();
    for (index, entry) in entries.iter().enumerate() {
        let context = format!("veil.baseline.json entries[{index}]");
        let entry = entry
            .as_object()
            .ok_or_else(|| VerifyError::SchemaViolation(format!("{context} must be an object")))?;
        let fingerprint = entry
            .get("fingerprint")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                VerifyError::SchemaViolation(format!("{context}.fingerprint must be a string"))
            })?;
        fingerprints.insert(fingerprint.to_string());
    }
    Ok(fingerprints)
}

fn validate_suppressed_findings_have_baseline_proof(
    report: &Value,
    baseline_fingerprints: Option<&HashSet<String>>,
) -> Result<(), VerifyError> {
    let findings = report
        .get("findings")
        .and_then(Value::as_array)
        .ok_or_else(|| {
            VerifyError::SchemaViolation("report.json findings must be an array".to_string())
        })?;

    for (index, finding) in findings.iter().enumerate() {
        let context = format!("report.json findings[{index}]");
        let baseline_status = finding
            .get("baselineStatus")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                VerifyError::SchemaViolation(format!("{context}.baselineStatus missing"))
            })?;
        if baseline_status != "suppressed" {
            continue;
        }

        let Some(baseline_fingerprints) = baseline_fingerprints else {
            return Err(VerifyError::SchemaViolation(format!(
                "{context}.baselineStatus suppressed requires veil.baseline.json baseline artifact"
            )));
        };
        let baseline_fingerprint = finding
            .get("baselineFingerprint")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                VerifyError::SchemaViolation(format!("{context}.baselineFingerprint missing"))
            })?;
        if !baseline_fingerprints.contains(baseline_fingerprint) {
            return Err(VerifyError::SchemaViolation(format!(
                "{context}.baselineFingerprint is not present in veil.baseline.json"
            )));
        }
    }
    Ok(())
}

fn validate_report_matches_run_meta(
    run_meta: &Value,
    report: &Value,
    baseline_fingerprints: Option<&HashSet<String>>,
) -> Result<(), VerifyError> {
    if run_meta.get("runId") != report.get("runId") {
        return Err(VerifyError::SchemaViolation(
            "run_meta.json runId does not match report.json runId".to_string(),
        ));
    }
    if run_meta.get("generatedAtUtc") != report.get("generatedAtUtc") {
        return Err(VerifyError::SchemaViolation(
            "run_meta.json generatedAtUtc does not match report.json generatedAtUtc".to_string(),
        ));
    }

    validate_report_summary_matches_findings(report)?;
    validate_suppressed_findings_have_baseline_proof(report, baseline_fingerprints)?;

    if run_meta.pointer("/result/summary") != report.get("summary") {
        return Err(VerifyError::SchemaViolation(
            "run_meta.json result.summary does not match report.json summary".to_string(),
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
    let mut extracted_sizes: HashMap<String, u64> = HashMap::new();
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

        if !is_allowed_evidence_pack_v1_file(&name) {
            return Err(VerifyError::SchemaViolation(format!(
                "Evidence Pack v1 contains unsupported file: {name}"
            )));
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
        extracted_sizes.insert(name.clone(), uncompressed_size);

        // Keep schema-bearing files in memory to parse them for structural validations
        if name == "run_meta.json"
            || name == "report.json"
            || name == crate::baseline::DEFAULT_BASELINE_FILE
        {
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
    validate_v1_run_meta(&run_meta)?;

    let limit_reached = run_meta
        .pointer("/result/limitReached")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);
    let result_status = run_meta
        .pointer("/result/status")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("error");
    let is_complete = run_meta
        .pointer("/result/summary/coverageComplete")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(!limit_reached)
        && !limit_reached
        && matches!(result_status, "success" | "violation");

    let findings_count = run_meta
        .pointer("/result/summary/effectiveFindings")
        .and_then(serde_json::Value::as_u64)
        .map(|count| count as usize)
        .unwrap_or(0);

    // 5. Match hashes against run_meta.json tracking
    let mut report_json_path = None;
    let baseline_file_present =
        extracted_files.contains_key(crate::baseline::DEFAULT_BASELINE_FILE);
    if let Some(artifacts_map) = run_meta.get("artifacts").and_then(Value::as_object) {
        let expected_files = [
            ("reportHtml", false),
            ("reportJson", false),
            ("effectiveConfig", false),
            ("baseline", true),
        ];
        for (camel_key, optional) in expected_files {
            let Some(art) = artifacts_map.get(camel_key) else {
                if optional {
                    if baseline_file_present {
                        return Err(VerifyError::SchemaViolation(
                            "artifact baseline missing from run_meta.json while veil.baseline.json is present"
                                .to_string(),
                        ));
                    }
                    continue;
                }
                return Err(VerifyError::SchemaViolation(format!(
                    "artifact {} missing from run_meta.json",
                    camel_key
                )));
            };
            if optional && art.is_null() {
                if baseline_file_present {
                    return Err(VerifyError::SchemaViolation(
                        "artifact baseline must describe veil.baseline.json when it is present"
                            .to_string(),
                    ));
                }
                continue;
            }

            let art = art.as_object().ok_or_else(|| {
                VerifyError::SchemaViolation(format!(
                    "artifact {} must be an object in run_meta.json",
                    camel_key
                ))
            })?;
            let expected_path = require_canonical_artifact_path(art, camel_key)?;
            let expected_hash = art.get("sha256").and_then(Value::as_str).ok_or_else(|| {
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
            if let Some(expected_size) = art.get("sizeBytes").and_then(Value::as_u64) {
                let actual_size = extracted_sizes
                    .get(expected_path)
                    .ok_or_else(|| VerifyError::MissingFile(expected_path.to_string()))?;
                if *actual_size != expected_size {
                    return Err(VerifyError::SchemaViolation(format!(
                        "artifact {camel_key} sizeBytes mismatch for {expected_path} (expected {expected_size}, got {actual_size})"
                    )));
                }
            }
            if camel_key == "reportJson" {
                report_json_path = Some(expected_path.to_string());
            }
        }
    } else {
        return Err(VerifyError::SchemaViolation(
            "artifacts map missing from run_meta.json".to_string(),
        ));
    }
    let report_json_path = report_json_path.ok_or_else(|| {
        VerifyError::SchemaViolation("artifact reportJson missing from run_meta.json".to_string())
    })?;
    let report_json_buf = extracted_content
        .get(&report_json_path)
        .ok_or_else(|| VerifyError::MissingFile(report_json_path.clone()))?;
    let report_json: Value = serde_json::from_slice(report_json_buf)?;
    let baseline_fingerprints = if baseline_file_present {
        let baseline_buf = extracted_content
            .get(crate::baseline::DEFAULT_BASELINE_FILE)
            .ok_or_else(|| {
                VerifyError::MissingFile(crate::baseline::DEFAULT_BASELINE_FILE.to_string())
            })?;
        Some(parse_baseline_fingerprints(baseline_buf)?)
    } else {
        None
    };
    validate_evidence_report(&report_json)?;
    validate_report_matches_run_meta(&run_meta, &report_json, baseline_fingerprints.as_ref())?;

    // 6. Validate Application Policies
    if options.require_complete && !is_complete {
        return Ok(VerifyResult {
            status: VerifyStatus::PolicyViolation,
            is_complete,
            findings_count,
            message:
                "Policy Violation: Evidence pack is incomplete (run status or coverage is incomplete)."
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
