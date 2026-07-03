use axum::{
    extract::{rejection::JsonRejection, Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use chrono::{Duration, Utc};
use std::collections::{BTreeMap, HashMap};
use std::path::{Path as FsPath, PathBuf};
use std::sync::Arc;
use tower_sessions::Session;
use veil_core::finding_id::SpanData;

#[allow(dead_code)]
pub mod dto;
pub use dto::*;

use crate::{config_loader::ConfigLayers, AppState};

type ApiErrorResponse = (StatusCode, Json<ErrorEnvelope>);

pub fn error_response(
    status: StatusCode,
    code: ErrorCode,
    message: impl Into<String>,
    next_action: Option<NextAction>,
) -> ApiErrorResponse {
    (
        status,
        Json(ErrorEnvelope {
            error: ErrorBody {
                code,
                message: message.into(),
                next_action,
            },
        }),
    )
}

fn json_rejection_response(rejection: JsonRejection) -> ApiErrorResponse {
    error_response(
        StatusCode::BAD_REQUEST,
        ErrorCode::InvalidRequest,
        format!("Invalid JSON request body: {rejection}"),
        None,
    )
}

// GET /api/me
// Returns the currently authenticated context (either SSO user or local token context).
pub async fn get_me(session: Session) -> Result<Json<AuthContext>, StatusCode> {
    if let Ok(Some(email)) = session.get::<String>("user_email").await {
        return Ok(Json(AuthContext::Sso(SsoAuthContext {
            authenticated: true,
            kind: AuthContextType::Sso,
            email,
            name: session.get::<String>("user_name").await.unwrap_or_default(),
            enterprise_opt_in: true,
        })));
    }

    Ok(Json(AuthContext::LocalToken(LocalTokenAuthContext {
        authenticated: true,
        kind: AuthContextType::LocalToken,
    })))
}

struct BaselineFileInfo {
    content: String,
    snapshot: veil_core::baseline::BaselineSnapshot,
}

enum BaselineFileError {
    PathDenied(String),
    InvalidRequest(String),
}

struct FindingBuckets {
    all: Vec<SafeFindingApiV1>,
    effective: Vec<SafeFindingApiV1>,
    suppressed: Vec<SafeFindingApiV1>,
}

struct AggregatedScan {
    findings: Vec<veil_core::Finding>,
    scanned_files: usize,
    skipped_files: usize,
    limit_reached: bool,
    limit_reasons: Vec<String>,
    builtin_skips: Vec<String>,
}

fn repo_root() -> PathBuf {
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

fn normalized_paths(paths: Option<Vec<String>>) -> Vec<String> {
    match paths {
        Some(paths) if !paths.is_empty() => paths,
        _ => vec![".".to_string()],
    }
}

fn preset_name(preset: PresetName) -> &'static str {
    match preset {
        PresetName::StandardJp => "standard-jp",
        PresetName::FintechJp => "fintech-jp",
        PresetName::GovJp => "gov-jp",
        PresetName::SiVendorJp => "si-vendor-jp",
        PresetName::LogsJp => "logs-jp",
    }
}

fn scan_path_for_request(requested: &str, safe_path: &FsPath) -> PathBuf {
    let requested_path = PathBuf::from(requested);
    if !requested_path.is_absolute() {
        return requested_path;
    }

    let root = repo_root();
    let root = std::fs::canonicalize(&root).unwrap_or(root);
    let safe_path = std::fs::canonicalize(safe_path).unwrap_or_else(|_| safe_path.to_path_buf());
    if let Ok(relative) = safe_path.strip_prefix(&root) {
        if relative.as_os_str().is_empty() {
            PathBuf::from(".")
        } else {
            relative.to_path_buf()
        }
    } else {
        safe_path
    }
}

fn config_error_response(error: impl std::fmt::Display) -> ApiErrorResponse {
    error_response(
        StatusCode::BAD_REQUEST,
        ErrorCode::InvalidRequest,
        format!("Failed to load configuration: {error}"),
        None,
    )
}

fn rules_error_response(error: impl std::fmt::Display) -> ApiErrorResponse {
    error_response(
        StatusCode::BAD_REQUEST,
        ErrorCode::InvalidRequest,
        format!("Failed to load rules: {error}"),
        None,
    )
}

fn load_rules_for_api(
    config: &veil_config::Config,
) -> Result<Vec<veil_core::Rule>, ApiErrorResponse> {
    load_rules_for_api_with_extra(config, Vec::new())
}

fn load_rules_for_api_with_extra(
    config: &veil_config::Config,
    extra_rules: Vec<veil_core::Rule>,
) -> Result<Vec<veil_core::Rule>, ApiErrorResponse> {
    veil_core::try_get_all_rules(config, extra_rules).map_err(rules_error_response)
}

fn load_rules_for_scan_with_preset(
    config: &veil_config::Config,
    preset: Option<PresetName>,
) -> Result<Vec<veil_core::Rule>, ApiErrorResponse> {
    let preset_name = preset.map(preset_name);
    let extra_rules = validate_preset_runtime_assets_for_api(config, preset_name)?;
    if extra_rules.is_empty() {
        return load_rules_for_api(config);
    }

    let mut config_without_rules_dir = config.clone();
    config_without_rules_dir.core.rules_dir = None;
    load_rules_for_api_with_extra(&config_without_rules_dir, extra_rules)
}

fn load_config_layers_for_api(_paths: &[String]) -> Result<ConfigLayers, ApiErrorResponse> {
    let repo_config = repo_root().join("veil.toml");
    load_config_layers_from_repo_config(repo_config)
}

fn load_config_layers_from_repo_config(
    repo_config: PathBuf,
) -> Result<ConfigLayers, ApiErrorResponse> {
    let explicit_path = repo_config.is_file().then_some(repo_config);
    crate::config_loader::load_config_layers(explicit_path.as_ref()).map_err(config_error_response)
}

fn load_effective_config_for_paths(
    paths: &[String],
) -> Result<veil_config::Config, ApiErrorResponse> {
    Ok(load_config_layers_for_api(paths)?.effective)
}

fn load_effective_config_for_paths_with_preset(
    paths: &[String],
    preset: Option<PresetName>,
) -> Result<veil_config::Config, ApiErrorResponse> {
    let config = load_effective_config_for_paths(paths)?;
    apply_request_preset_for_api(config, preset)
}

fn apply_request_preset_for_api(
    config: veil_config::Config,
    preset: Option<PresetName>,
) -> Result<veil_config::Config, ApiErrorResponse> {
    let Some(preset) = preset else {
        return Ok(config);
    };
    let preset_name = preset_name(preset);
    let config = veil_config::apply_builtin_preset_as_base(config, preset_name)
        .map_err(config_error_response)?;

    Ok(config)
}

fn validate_preset_runtime_assets_for_api(
    config: &veil_config::Config,
    preset_name: Option<&str>,
) -> Result<Vec<veil_core::Rule>, ApiErrorResponse> {
    if preset_name == Some("logs-jp") {
        return validate_logs_preset_rule_pack_for_api(config);
    }

    Ok(Vec::new())
}

fn logs_preset_config_error(message: impl Into<String>) -> ApiErrorResponse {
    error_response(
        StatusCode::BAD_REQUEST,
        ErrorCode::InvalidRequest,
        message,
        None,
    )
}

fn validate_logs_preset_rule_pack_for_api(
    config: &veil_config::Config,
) -> Result<Vec<veil_core::Rule>, ApiErrorResponse> {
    let Some(rules_dir) = &config.core.rules_dir else {
        return Err(logs_preset_config_error(
            "Preset 'logs-jp' requires the log rule pack. Run `veil init --preset logs-jp` or set [core] rules_dir = \"rules/log\".",
        ));
    };

    let path = FsPath::new(rules_dir);
    if !path.is_dir() {
        return Err(logs_preset_config_error(format!(
            "Preset 'logs-jp' requires the log rule pack at {}. Run `veil init --preset logs-jp` or set [core] rules_dir = \"rules/log\".",
            path.display()
        )));
    }

    let rules = veil_core::rules::pack::load_rule_pack(path).map_err(|err| {
        logs_preset_config_error(format!(
            "Preset 'logs-jp' requires a valid log rule pack at {}: {err}",
            path.display()
        ))
    })?;
    let missing_ids: Vec<_> = veil_config::LOGS_JP_REQUIRED_RULE_IDS
        .iter()
        .copied()
        .filter(|required_id| !rules.iter().any(|rule| rule.id == *required_id))
        .collect();
    if !missing_ids.is_empty() {
        return Err(logs_preset_config_error(format!(
            "Preset 'logs-jp' requires a log rule pack containing these rules at {}: {}. Run `veil init --preset logs-jp` or set [core] rules_dir = \"rules/log\".",
            path.display(),
            missing_ids.join(", ")
        )));
    }

    Ok(rules)
}

fn baseline_file_error_response(error: BaselineFileError) -> ApiErrorResponse {
    match error {
        BaselineFileError::PathDenied(error) => error_response(
            StatusCode::FORBIDDEN,
            ErrorCode::PathDenied,
            format!("Path denied: {error}"),
            Some(NextAction::NarrowScope),
        ),
        BaselineFileError::InvalidRequest(error) => error_response(
            StatusCode::BAD_REQUEST,
            ErrorCode::InvalidRequest,
            error,
            None,
        ),
    }
}

fn resolve_baseline_file(
    requested_path: Option<&str>,
) -> Result<Option<BaselineFileInfo>, BaselineFileError> {
    let root = repo_root();
    resolve_baseline_file_from_root(&root, requested_path)
}

fn resolve_baseline_file_from_root(
    root: &FsPath,
    requested_path: Option<&str>,
) -> Result<Option<BaselineFileInfo>, BaselineFileError> {
    if let Some(requested_path) = requested_path {
        let path = validate_safe_path(requested_path).map_err(BaselineFileError::PathDenied)?;
        let content = std::fs::read_to_string(&path).map_err(|err| {
            BaselineFileError::InvalidRequest(format!("Failed to read baselineFile: {err}"))
        })?;
        let snapshot = veil_core::baseline::load_baseline(&path).map_err(|err| {
            BaselineFileError::InvalidRequest(format!("Failed to load baselineFile: {err}"))
        })?;

        return Ok(Some(BaselineFileInfo { content, snapshot }));
    }

    let Some(path) = veil_core::baseline::resolve_compatible_baseline_path(root) else {
        return Ok(None);
    };
    let content = std::fs::read_to_string(&path).map_err(|err| {
        BaselineFileError::InvalidRequest(format!(
            "Failed to read discovered baselineFile {}: {err}",
            path.display()
        ))
    })?;
    let snapshot = veil_core::baseline::load_baseline(&path).map_err(|err| {
        BaselineFileError::InvalidRequest(format!(
            "Failed to load discovered baselineFile {}: {err}",
            path.display()
        ))
    })?;

    Ok(Some(BaselineFileInfo { content, snapshot }))
}

fn rule_lookup(rules: &[veil_core::Rule]) -> HashMap<String, (String, Vec<String>)> {
    rules
        .iter()
        .map(|rule| (rule.id.clone(), (rule.category.clone(), rule.tags.clone())))
        .collect()
}

fn finding_id(finding: &veil_core::Finding, ordinal: usize) -> String {
    let span = SpanData {
        start_line: finding.line_number as u64,
        start_col: 0,
        end_line: finding.line_number as u64,
        end_col: 0,
    };
    let public_discriminator = format!("{}|{}", finding.masked_snippet, ordinal);
    veil_core::FindingId::new(
        &finding.rule_id,
        &finding.path,
        &span,
        &public_discriminator,
    )
    .to_string()
}

fn to_safe_finding(
    finding: &veil_core::Finding,
    baseline_status: BaselineStatus,
    rules: &HashMap<String, (String, Vec<String>)>,
    ordinal: usize,
) -> SafeFindingApiV1 {
    let (category, tags) = rules
        .get(&finding.rule_id)
        .cloned()
        .unwrap_or_else(|| ("uncategorized".to_string(), Vec::new()));

    SafeFindingApiV1 {
        finding_id: finding_id(finding, ordinal),
        baseline_fingerprint: veil_core::baseline::generate_fingerprint(finding),
        path: finding.path.to_string_lossy().to_string(),
        line_number: finding.line_number,
        rule_id: finding.rule_id.clone(),
        severity: SeverityName::from(&finding.severity),
        score: finding.score,
        grade: GradeName::from(&finding.grade),
        masked_snippet: finding.masked_snippet.clone(),
        category,
        tags,
        baseline_status,
    }
}

fn count_severities(findings: &[SafeFindingApiV1]) -> SeverityCounts {
    let mut counts = SeverityCounts::zero();
    for finding in findings {
        counts.increment(finding.severity);
    }
    counts
}

fn bucket_findings(
    findings: Vec<veil_core::Finding>,
    baseline: Option<&veil_core::baseline::BaselineSnapshot>,
    rules: &HashMap<String, (String, Vec<String>)>,
) -> FindingBuckets {
    let known_fingerprints = baseline.map(veil_core::baseline::BaselineSnapshot::fingerprint_set);
    let mut all = Vec::new();
    let mut effective = Vec::new();
    let mut suppressed = Vec::new();

    for (ordinal, finding) in findings.iter().enumerate() {
        let baseline_status = match &known_fingerprints {
            Some(fingerprints)
                if fingerprints.contains(&veil_core::baseline::generate_fingerprint(finding)) =>
            {
                BaselineStatus::Suppressed
            }
            Some(_) => BaselineStatus::New,
            None => BaselineStatus::None,
        };
        let safe = to_safe_finding(finding, baseline_status, rules, ordinal);
        if matches!(baseline_status, BaselineStatus::Suppressed) {
            suppressed.push(safe.clone());
        } else {
            effective.push(safe.clone());
        }
        all.push(safe);
    }

    FindingBuckets {
        all,
        effective,
        suppressed,
    }
}

fn build_evidence_summary(buckets: &FindingBuckets, coverage_complete: bool) -> EvidenceSummary {
    EvidenceSummary {
        total_findings: buckets.all.len(),
        suppressed_findings: buckets.suppressed.len(),
        effective_findings: buckets.effective.len(),
        severity_counts: count_severities(&buckets.effective),
        all_severity_counts: count_severities(&buckets.all),
        suppressed_severity_counts: count_severities(&buckets.suppressed),
        coverage_complete,
    }
}

fn policy_response_from_layers(layers: ConfigLayers) -> Result<PolicyResponse, ApiErrorResponse> {
    let config = layers.effective.clone();
    let rules = load_rules_for_api(&config)?;
    let repo_config_path = repo_root().join("veil.toml");
    let org_config_path = std::env::var("VEIL_ORG_CONFIG").ok();

    Ok(PolicyResponse {
        schema_version: LocalApiSchemaVersion::VeilProLocalApiV1,
        has_org_config: layers.org.is_some(),
        org_config_path,
        repo_config_path: repo_config_path
            .exists()
            .then(|| repo_config_path.to_string_lossy().to_string()),
        effective_rules_count: rules.len(),
        preset: None,
        layers: vec![
            ConfigLayerSummary {
                name: ConfigLayerName::Builtin,
                path: None,
                loaded: true,
                warnings: Vec::new(),
            },
            ConfigLayerSummary {
                name: ConfigLayerName::Org,
                path: std::env::var("VEIL_ORG_CONFIG").ok(),
                loaded: layers.org.is_some(),
                warnings: Vec::new(),
            },
            ConfigLayerSummary {
                name: ConfigLayerName::Repo,
                path: repo_config_path
                    .exists()
                    .then(|| repo_config_path.to_string_lossy().to_string()),
                loaded: repo_config_path.exists(),
                warnings: Vec::new(),
            },
        ],
        conflicts: Vec::new(),
    })
}

fn policy_response() -> Result<PolicyResponse, ApiErrorResponse> {
    policy_response_from_layers(load_config_layers_for_api(&[".".to_string()])?)
}

fn limit_reasons_for(result: &veil_core::ScanResult) -> Vec<String> {
    let mut reasons = Vec::new();
    if result.file_limit_reached {
        reasons.push("file-limit".to_string());
    }
    if result.max_file_size_reached {
        reasons.push("max-file-size".to_string());
    }
    if result.read_error_reached {
        reasons.push("read-error".to_string());
    }
    if result.limit_reached && !result.file_limit_reached && !result.findings.is_empty() {
        reasons.push("result-limit".to_string());
    }
    reasons
}

fn severity_score_threshold(severity: SeverityName) -> u32 {
    let severity = match severity {
        SeverityName::Low => veil_core::Severity::Low,
        SeverityName::Medium => veil_core::Severity::Medium,
        SeverityName::High => veil_core::Severity::High,
        SeverityName::Critical => veil_core::Severity::Critical,
    };
    veil_core::severity_min_score(&severity)
}

fn policy_violated(findings: &[SafeFindingApiV1], req: &ScanRequest) -> bool {
    let has_thresholds = req.fail_on_findings.is_some()
        || req.fail_on_score.is_some()
        || req.fail_on_severity.is_some();
    let findings_threshold = req
        .fail_on_findings
        .is_some_and(|threshold| findings.len() >= threshold);
    let score_threshold = req
        .fail_on_score
        .is_some_and(|threshold| findings.iter().any(|finding| finding.score >= threshold));
    let severity_threshold = req.fail_on_severity.is_some_and(|threshold| {
        let threshold_score = severity_score_threshold(threshold);
        findings
            .iter()
            .any(|finding| finding.score >= threshold_score)
    });

    if has_thresholds {
        findings_threshold || score_threshold || severity_threshold
    } else {
        !findings.is_empty()
    }
}

fn scan_paths_with_global_limit(
    paths_to_scan: Vec<String>,
    rules: &[veil_core::Rule],
    config: &veil_config::Config,
) -> Result<AggregatedScan, ApiErrorResponse> {
    let mut findings = Vec::new();
    let mut scanned_files = 0;
    let mut skipped_files = 0;
    let mut limit_reached = false;
    let mut limit_reasons = Vec::new();
    let mut builtin_skips = Vec::new();
    let max_findings = config.output.max_findings;
    let mut raw_findings_count = 0usize;

    for path in paths_to_scan {
        if max_findings.is_some_and(|max| raw_findings_count >= max) {
            limit_reached = true;
            limit_reasons.push("result-limit".to_string());
            break;
        }
        let safe_path = validate_safe_path(&path).map_err(|error| {
            error_response(
                StatusCode::FORBIDDEN,
                ErrorCode::PathDenied,
                format!("Path denied: {error}"),
                Some(NextAction::NarrowScope),
            )
        })?;
        let scan_path = scan_path_for_request(&path, &safe_path);
        let mut run_config = config.clone();
        if let Some(max) = max_findings {
            run_config.output.max_findings = Some(max.saturating_sub(raw_findings_count));
        }
        let result = veil_core::scan_path(&scan_path, rules, &run_config);
        scanned_files += result.scanned_files;
        skipped_files += result.skipped_files;
        let real_finding_limit_reached = result.limit_reached && !result.findings.is_empty();
        limit_reached |= real_finding_limit_reached
            || result.file_limit_reached
            || result.max_file_size_reached
            || result.read_error_reached;
        limit_reasons.extend(limit_reasons_for(&result));
        builtin_skips.extend(result.builtin_skips);
        raw_findings_count += result.findings.len();
        findings.extend(result.findings);
    }

    limit_reasons.sort();
    limit_reasons.dedup();
    builtin_skips.sort();
    builtin_skips.dedup();

    Ok(AggregatedScan {
        findings,
        scanned_files,
        skipped_files,
        limit_reached,
        limit_reasons,
        builtin_skips,
    })
}

// --- Endpoints ---

pub async fn list_projects(State(_state): State<Arc<AppState>>) -> Json<ProjectsResponse> {
    let current_dir = repo_root();
    let current_dir_text = current_dir.to_string_lossy().to_string();
    let display_name = current_dir
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| current_dir_text.clone());

    Json(ProjectsResponse {
        schema_version: LocalApiSchemaVersion::VeilProLocalApiV1,
        current_dir: current_dir_text.clone(),
        projects: vec![ProjectSummary {
            id: current_dir_text.clone(),
            display_name,
            root_path: current_dir_text,
            is_current: true,
            has_repo_config: repo_root().join("veil.toml").exists(),
        }],
    })
}

pub async fn scan_project(
    State(state): State<Arc<AppState>>,
    request: Result<Json<ScanRequest>, JsonRejection>,
) -> Result<Json<ScanResponse>, ApiErrorResponse> {
    let Json(req) = request.map_err(json_rejection_response)?;
    let paths_to_scan = normalized_paths(req.paths.clone());
    if let Some(ScanMode::Staged | ScanMode::Ci) = req.mode {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            ErrorCode::InvalidRequest,
            "mode staged/ci is not implemented by the Local API in PR-0; use mode full or omit mode.",
            None,
        ));
    }
    if req.fail_on_findings == Some(0) {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            ErrorCode::InvalidRequest,
            "failOnFindings must be >= 1",
            None,
        ));
    }
    if req.fail_on_score.is_some_and(|score| score > 100) {
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            ErrorCode::InvalidRequest,
            "failOnScore must be between 0 and 100",
            None,
        ));
    }

    let config = load_effective_config_for_paths_with_preset(&paths_to_scan, req.preset)?;
    let rules = load_rules_for_scan_with_preset(&config, req.preset)?;
    let rules_by_id = rule_lookup(&rules);
    let baseline = resolve_baseline_file(req.baseline_file.as_deref())
        .map_err(baseline_file_error_response)?;

    let aggregate = scan_paths_with_global_limit(paths_to_scan, &rules, &config)?;

    let buckets = bucket_findings(
        aggregate.findings,
        baseline.as_ref().map(|info| &info.snapshot),
        &rules_by_id,
    );
    let coverage_complete = !aggregate.limit_reached;
    let summary = build_evidence_summary(&buckets, coverage_complete);
    let status = if aggregate.limit_reached {
        RunStatus::Incomplete
    } else if policy_violated(&buckets.effective, &req) {
        RunStatus::Violation
    } else {
        RunStatus::Success
    };

    let response_findings = if req.include_suppressed {
        buckets.all.clone()
    } else {
        buckets.effective.clone()
    };

    let (run_meta, cached_run) = crate::evidence_generator::generate_evidence_pack(
        &config,
        &buckets.all,
        summary.clone(),
        status,
        aggregate.limit_reached,
        aggregate.limit_reasons.clone(),
        aggregate.scanned_files,
        aggregate.skipped_files,
        150,
        baseline.map(|info| info.content),
    );
    let run_id = run_meta.run_id.clone();

    if !state
        .run_cache
        .write()
        .await
        .insert(run_id.clone(), cached_run)
    {
        return Err(error_response(
            StatusCode::PAYLOAD_TOO_LARGE,
            ErrorCode::RunTooLarge,
            "Run evidence is too large to cache. Narrow the scan scope or reduce findings before exporting evidence.",
            Some(NextAction::NarrowScope),
        ));
    }

    Ok(Json(ScanResponse {
        schema_version: LocalApiSchemaVersion::VeilProLocalApiV1,
        run_id,
        status,
        scanned_files: aggregate.scanned_files,
        skipped_files: aggregate.skipped_files,
        total_findings: summary.total_findings,
        suppressed_findings: summary.suppressed_findings,
        effective_findings: summary.effective_findings,
        coverage_complete,
        severity_counts: summary.severity_counts,
        all_severity_counts: summary.all_severity_counts,
        suppressed_severity_counts: summary.suppressed_severity_counts,
        limit_reached: aggregate.limit_reached,
        limit_reasons: aggregate.limit_reasons,
        builtin_skips: aggregate.builtin_skips,
        findings: response_findings,
        expires_at_utc: (Utc::now() + Duration::minutes(30)).to_rfc3339(),
    }))
}

pub async fn get_policy(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<PolicyResponse>, ApiErrorResponse> {
    Ok(Json(policy_response()?))
}

pub async fn get_doctor(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<DoctorResponse>, ApiErrorResponse> {
    let layers = load_config_layers_for_api(&[".".to_string()])?;
    let config = layers.effective.clone();
    let mut bounds = BTreeMap::new();
    bounds.insert(
        "maxFileCount".to_string(),
        BoundValue::Number(
            config
                .core
                .max_file_count
                .unwrap_or(veil_core::DEFAULT_MAX_FILE_COUNT) as u64,
        ),
    );
    bounds.insert(
        "maxFileSizeBytes".to_string(),
        BoundValue::Number(
            config
                .core
                .max_file_size
                .unwrap_or(veil_core::DEFAULT_MAX_FILE_SIZE_BYTES),
        ),
    );
    bounds.insert(
        "maxFindings".to_string(),
        BoundValue::Number(config.output.max_findings.unwrap_or(1_000) as u64),
    );

    Ok(Json(DoctorResponse {
        schema_version: LocalApiSchemaVersion::VeilProLocalApiV1,
        product_version: env!("CARGO_PKG_VERSION").to_string(),
        os: std::env::consts::OS.to_string(),
        rust_version: option_env!("RUSTC_VERSION").map(str::to_string),
        config: policy_response_from_layers(layers)?,
        bounds,
        rule_packs: vec![DoctorRulePack {
            name: "default".to_string(),
            version: None,
            source: RulePackSource::Embedded,
        }],
        network_mode: NetworkMode::LocalOnly,
        warnings: Vec::new(),
    }))
}

pub async fn write_baseline(
    State(_state): State<Arc<AppState>>,
    request: Result<Json<BaselineRequest>, JsonRejection>,
) -> Result<Json<BaselineResponse>, ApiErrorResponse> {
    let Json(req) = request.map_err(json_rejection_response)?;
    let paths_to_scan = normalized_paths(req.paths);
    let config = load_effective_config_for_paths(&paths_to_scan)?;
    let rules = load_rules_for_api(&config)?;
    let aggregate = scan_paths_with_global_limit(paths_to_scan, &rules, &config)?;
    if aggregate.limit_reached {
        let reasons = if aggregate.limit_reasons.is_empty() {
            "unknown".to_string()
        } else {
            aggregate.limit_reasons.join(", ")
        };
        return Err(error_response(
            StatusCode::BAD_REQUEST,
            ErrorCode::InvalidRequest,
            format!("Cannot write baseline from incomplete scan: {reasons}"),
            Some(NextAction::NarrowScope),
        ));
    }

    let tool_version = env!("CARGO_PKG_VERSION");
    let snapshot = veil_core::baseline::from_findings(&aggregate.findings, tool_version);
    let output_path = if let Some(output_path) = req.output_path {
        validate_safe_path(&output_path).map_err(|error| {
            error_response(
                StatusCode::FORBIDDEN,
                ErrorCode::PathDenied,
                error,
                Some(NextAction::NarrowScope),
            )
        })?
    } else {
        veil_core::baseline::default_baseline_path(&repo_root())
    };

    veil_core::baseline::save_baseline(&output_path, &snapshot).map_err(|err| {
        error_response(
            StatusCode::BAD_REQUEST,
            ErrorCode::InvalidRequest,
            format!("Failed to write baseline: {err}"),
            Some(NextAction::NarrowScope),
        )
    })?;

    Ok(Json(BaselineResponse {
        schema_version: LocalApiSchemaVersion::VeilProLocalApiV1,
        file_path: output_path.to_string_lossy().to_string(),
        findings_count: snapshot.entries.len(),
        written: true,
        next_action: NextAction::CommitBaseline,
    }))
}

/// B2B Security: Ensure path does not escape the current project directory using traversal ('../')
/// or absolute paths that point outside.
fn validate_safe_path(p: &str) -> Result<PathBuf, String> {
    if p.contains("..") {
        return Err("Path traversal '..' is explicitly forbidden.".to_string());
    }

    let path = PathBuf::from(p);
    let current_dir =
        std::env::current_dir().map_err(|e| format!("Cannot read current dir: {}", e))?;

    let resolved = if path.is_absolute() {
        path.clone()
    } else {
        current_dir.join(&path)
    };

    let check_path = if resolved.exists() {
        resolved.clone()
    } else {
        resolved
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or(resolved.clone())
    };

    if check_path.exists() {
        if let Ok(canon_target) = std::fs::canonicalize(&check_path) {
            if let Ok(canon_curr) = std::fs::canonicalize(&current_dir) {
                if !canon_target.starts_with(&canon_curr) {
                    return Err(format!("Arbitrary path access blocked: {}", p));
                }
            }
        }
    } else if path.is_absolute() && !resolved.starts_with(&current_dir) {
        return Err(format!(
            "Absolute path outside working directory blocked: {}",
            p
        ));
    }

    Ok(resolved)
}

pub async fn get_run_meta(
    State(state): State<Arc<AppState>>,
    Path(run_id): Path<String>,
) -> Result<Json<crate::evidence::RunMeta>, ApiErrorResponse> {
    let mut cache = state.run_cache.write().await;
    if let Some(cached) = cache.get(&run_id) {
        Ok(Json(cached.meta))
    } else {
        Err(error_response(
            StatusCode::GONE,
            ErrorCode::RunExpired,
            "Run evidence has expired or runId is invalid. Please trigger a new scan.",
            Some(NextAction::Rescan),
        ))
    }
}

pub async fn export_evidence(
    State(state): State<Arc<AppState>>,
    Path(run_id): Path<String>,
) -> Result<impl IntoResponse, ApiErrorResponse> {
    let mut cache = state.run_cache.write().await;
    let cached = match cache.get(&run_id) {
        Some(c) => c,
        None => {
            return Err(error_response(
                StatusCode::GONE,
                ErrorCode::RunExpired,
                "Run evidence has expired or runId is invalid. Please trigger a new scan.",
                Some(NextAction::Rescan),
            ));
        }
    };

    let mut zip_data = Vec::new();
    {
        let mut zip = zip::ZipWriter::new(std::io::Cursor::new(&mut zip_data));
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated)
            .unix_permissions(0o644);

        let _ = zip.start_file("report.html", options);
        let _ = std::io::Write::write_all(&mut zip, cached.report_html.as_bytes());

        let _ = zip.start_file("report.json", options);
        let _ = std::io::Write::write_all(&mut zip, cached.report_json.as_bytes());

        let _ = zip.start_file("effective_config.toml", options);
        let _ = std::io::Write::write_all(&mut zip, cached.effective_config.as_bytes());

        let _ = zip.start_file("run_meta.json", options);
        let meta_json = serde_json::to_string_pretty(&cached.meta).unwrap();
        let _ = std::io::Write::write_all(&mut zip, meta_json.as_bytes());

        if let Some(baseline) = &cached.baseline_json {
            let _ = zip.start_file(veil_core::baseline::DEFAULT_BASELINE_FILE, options);
            let _ = std::io::Write::write_all(&mut zip, baseline.as_bytes());
        }

        let _ = zip.finish().map_err(|_| {
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                ErrorCode::InternalError,
                "Failed to assemble Evidence Pack ZIP archive.",
                None,
            )
        })?;
    }

    let headers = [
        (header::CONTENT_TYPE, "application/zip".to_string()),
        (
            header::CONTENT_DISPOSITION,
            format!("attachment; filename=\"veil-evidence-{}.zip\"", run_id),
        ),
    ];

    Ok((headers, zip_data))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_target_dir(name: &str) -> PathBuf {
        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = repo_root().join("target").join(format!(
            "veil-pro-api-{name}-{}-{suffix}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn policy_finding(score: u32, severity: SeverityName) -> SafeFindingApiV1 {
        SafeFindingApiV1 {
            finding_id: format!("fx_{score}"),
            baseline_fingerprint: format!("sha256:{score:064x}"),
            path: "src/config.rs".to_string(),
            line_number: 1,
            rule_id: "creds.test".to_string(),
            severity,
            score,
            grade: GradeName::High,
            masked_snippet: "token = <REDACTED>".to_string(),
            category: "credentials".to_string(),
            tags: Vec::new(),
            baseline_status: BaselineStatus::New,
        }
    }

    #[test]
    fn test_validate_safe_path() {
        assert!(validate_safe_path(".").is_ok());
        assert!(validate_safe_path("./src").is_ok());

        assert!(validate_safe_path("..").is_err());
        assert!(validate_safe_path("../some_other_folder").is_err());
        assert!(validate_safe_path("src/../..").is_err());

        #[cfg(unix)]
        assert!(validate_safe_path("/etc/passwd").is_err());
        #[cfg(windows)]
        assert!(validate_safe_path("C:\\Windows\\System32").is_err());
    }

    #[test]
    fn scan_request_paths_missing_or_empty_defaults_to_current_directory() {
        assert_eq!(normalized_paths(None), vec![".".to_string()]);
        assert_eq!(normalized_paths(Some(Vec::new())), vec![".".to_string()]);
    }

    #[test]
    fn scan_path_preserves_relative_request_for_baseline_matching() {
        let safe_path = repo_root().join(".");

        assert_eq!(scan_path_for_request(".", &safe_path), PathBuf::from("."));
    }

    #[test]
    fn scan_path_converts_absolute_repo_root_to_relative() {
        let root = repo_root();
        let root_text = root.to_string_lossy().to_string();

        assert_eq!(scan_path_for_request(&root_text, &root), PathBuf::from("."));
    }

    #[test]
    fn safe_finding_exposes_distinct_finding_id_and_baseline_fingerprint() {
        let finding = veil_core::Finding {
            path: "src/config.rs".into(),
            line_number: 7,
            line_content: "token = secret".to_string(),
            rule_id: "creds.test".to_string(),
            matched_content: "secret".to_string(),
            masked_snippet: "token = <REDACTED>".to_string(),
            severity: veil_core::Severity::High,
            score: 80,
            grade: veil_core::Grade::High,
            span: Default::default(),
            utf16_range: Default::default(),
            context_before: Vec::new(),
            context_after: Vec::new(),
            commit_sha: None,
            author: None,
            date: None,
        };
        let safe = to_safe_finding(&finding, BaselineStatus::None, &HashMap::new(), 0);

        assert_ne!(safe.finding_id, safe.baseline_fingerprint);
        assert!(safe.finding_id.starts_with("fx_"));
        assert!(safe.baseline_fingerprint.starts_with("sha256:"));
    }

    #[test]
    fn safe_finding_id_does_not_depend_on_raw_match() {
        let mut finding = veil_core::Finding {
            path: "src/config.rs".into(),
            line_number: 7,
            line_content: "token = secret".to_string(),
            rule_id: "creds.test".to_string(),
            matched_content: "secret-one".to_string(),
            masked_snippet: "token = <REDACTED>".to_string(),
            severity: veil_core::Severity::High,
            score: 80,
            grade: veil_core::Grade::High,
            span: Default::default(),
            utf16_range: Default::default(),
            context_before: Vec::new(),
            context_after: Vec::new(),
            commit_sha: None,
            author: None,
            date: None,
        };
        let first = to_safe_finding(&finding, BaselineStatus::None, &HashMap::new(), 0);
        finding.matched_content = "secret-two".to_string();
        let second = to_safe_finding(&finding, BaselineStatus::None, &HashMap::new(), 0);
        let next_ordinal = to_safe_finding(&finding, BaselineStatus::None, &HashMap::new(), 1);

        assert_eq!(first.finding_id, second.finding_id);
        assert_ne!(first.finding_id, next_ordinal.finding_id);
    }

    #[test]
    fn fail_on_findings_uses_effective_findings_threshold() {
        let req = ScanRequest {
            fail_on_findings: Some(2),
            ..ScanRequest::default()
        };
        let one_finding = vec![policy_finding(80, SeverityName::High)];
        let two_findings = vec![
            policy_finding(80, SeverityName::High),
            policy_finding(70, SeverityName::Medium),
        ];

        assert!(!policy_violated(&one_finding, &req));
        assert!(policy_violated(&two_findings, &req));
    }

    #[test]
    fn fail_on_score_and_severity_respect_thresholds() {
        let findings = vec![policy_finding(60, SeverityName::Medium)];
        let below_low = vec![policy_finding(19, SeverityName::Critical)];
        let low_score = vec![policy_finding(20, SeverityName::Low)];
        let boosted_medium = vec![policy_finding(70, SeverityName::Medium)];
        let lowered_high = vec![policy_finding(60, SeverityName::High)];

        assert!(!policy_violated(
            &findings,
            &ScanRequest {
                fail_on_score: Some(80),
                ..ScanRequest::default()
            }
        ));
        assert!(policy_violated(
            &findings,
            &ScanRequest {
                fail_on_score: Some(60),
                ..ScanRequest::default()
            }
        ));
        assert!(!policy_violated(
            &findings,
            &ScanRequest {
                fail_on_severity: Some(SeverityName::High),
                ..ScanRequest::default()
            }
        ));
        assert!(policy_violated(
            &findings,
            &ScanRequest {
                fail_on_severity: Some(SeverityName::Medium),
                ..ScanRequest::default()
            }
        ));
        assert!(policy_violated(
            &low_score,
            &ScanRequest {
                fail_on_severity: Some(SeverityName::Low),
                ..ScanRequest::default()
            }
        ));
        assert!(!policy_violated(
            &below_low,
            &ScanRequest {
                fail_on_severity: Some(SeverityName::Low),
                ..ScanRequest::default()
            }
        ));
        assert!(policy_violated(
            &boosted_medium,
            &ScanRequest {
                fail_on_severity: Some(SeverityName::High),
                ..ScanRequest::default()
            }
        ));
        assert!(!policy_violated(
            &lowered_high,
            &ScanRequest {
                fail_on_severity: Some(SeverityName::High),
                ..ScanRequest::default()
            }
        ));
    }

    #[test]
    fn file_limit_has_dedicated_limit_reason() {
        let result = veil_core::ScanResult {
            file_limit_reached: true,
            ..veil_core::ScanResult::default()
        };

        assert_eq!(limit_reasons_for(&result), vec!["file-limit".to_string()]);
    }

    #[test]
    fn max_file_size_skip_has_dedicated_limit_reason() {
        let result = veil_core::ScanResult {
            max_file_size_reached: true,
            ..veil_core::ScanResult::default()
        };

        assert_eq!(
            limit_reasons_for(&result),
            vec!["max-file-size".to_string()]
        );
    }

    #[test]
    fn read_error_has_dedicated_limit_reason() {
        let result = veil_core::ScanResult {
            read_error_reached: true,
            ..veil_core::ScanResult::default()
        };

        assert_eq!(limit_reasons_for(&result), vec!["read-error".to_string()]);
    }

    #[test]
    fn invalid_repo_config_returns_error_instead_of_defaulting() {
        let dir = unique_target_dir("invalid-config");
        let config_path = dir.join("veil.toml");
        std::fs::write(&config_path, "[output]\nmax_findings = 0\n").unwrap();

        let (status, Json(body)) = load_config_layers_from_repo_config(config_path).unwrap_err();

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(matches!(body.error.code, ErrorCode::InvalidRequest));
        assert!(body
            .error
            .message
            .contains("Invalid config field 'output.max_findings'"));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn invalid_rule_validator_returns_error_instead_of_empty_rules() {
        let mut config = veil_config::Config::default();
        config.rules.insert(
            "custom.invalid_validator".to_string(),
            veil_config::RuleConfig {
                enabled: true,
                enabled_is_set: true,
                severity: None,
                pattern: Some("SECRET".to_string()),
                score: None,
                category: None,
                tags: None,
                base_score: None,
                context_lines_before: None,
                context_lines_after: None,
                validator: Some("unknown_validator".to_string()),
                description: None,
                placeholder: None,
            },
        );

        let (status, Json(body)) = load_rules_for_api(&config).unwrap_err();

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(matches!(body.error.code, ErrorCode::InvalidRequest));
        assert!(body.error.message.contains("Failed to load rules"));
        assert!(body
            .error
            .message
            .contains("Unknown validator 'unknown_validator'"));
    }

    #[test]
    fn local_api_preset_names_match_builtin_preset_ids() {
        let api_ids: std::collections::BTreeSet<_> = [
            PresetName::StandardJp,
            PresetName::FintechJp,
            PresetName::GovJp,
            PresetName::SiVendorJp,
            PresetName::LogsJp,
        ]
        .into_iter()
        .map(preset_name)
        .collect();
        let builtin_ids: std::collections::BTreeSet<_> =
            veil_config::BUILTIN_PRESET_IDS.iter().copied().collect();

        assert_eq!(api_ids, builtin_ids);
    }

    #[test]
    fn local_api_fintech_preset_applies_base_score_override() {
        let config = apply_request_preset_for_api(
            veil_config::Config::default(),
            Some(PresetName::FintechJp),
        )
        .unwrap();
        let rules = load_rules_for_api(&config).unwrap();
        let card_rule = rules
            .iter()
            .find(|rule| rule.id == "pii.fin.credit_card.keyword")
            .unwrap();

        assert_eq!(card_rule.base_score, Some(85));
    }

    #[test]
    fn discovered_invalid_baseline_returns_error() {
        let root = unique_target_dir("invalid-discovered-baseline");
        std::fs::write(
            root.join(veil_core::baseline::DEFAULT_BASELINE_FILE),
            "{bad json",
        )
        .unwrap();

        let result = resolve_baseline_file_from_root(&root, None);

        match result {
            Err(BaselineFileError::InvalidRequest(message)) => {
                assert!(message.contains("Failed to load discovered baselineFile"));
            }
            _ => panic!("expected invalid discovered baseline to return InvalidRequest"),
        }
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn binary_skips_do_not_trigger_result_limit() {
        let root = unique_target_dir("binary-skip-limit");
        let binary_path = root.join("asset.bin");
        std::fs::write(&binary_path, [b'a', 0, b'b']).unwrap();
        let repo = repo_root();
        let path = root
            .strip_prefix(&repo)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let mut config = veil_config::Config::default();
        config.output.max_findings = Some(1);
        let rules = veil_core::get_all_rules(&config, vec![]);

        let aggregate = scan_paths_with_global_limit(vec![path], &rules, &config).unwrap();

        assert!(!aggregate.limit_reached);
        assert_eq!(aggregate.skipped_files, 1);
        assert!(aggregate.findings.is_empty());
        assert!(!aggregate
            .limit_reasons
            .contains(&"result-limit".to_string()));
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn scan_paths_share_result_limit_across_requested_paths() {
        let root = unique_target_dir("global-limit");
        let first = root.join("first");
        let second = root.join("second");
        std::fs::create_dir_all(&first).unwrap();
        std::fs::create_dir_all(&second).unwrap();
        std::fs::write(first.join("secret.txt"), "aws_key = AKIA1234567890123456\n").unwrap();
        std::fs::write(
            second.join("secret.txt"),
            "aws_key = AKIA9999999999999999\n",
        )
        .unwrap();

        let repo = repo_root();
        let first_path = first
            .strip_prefix(&repo)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let second_path = second
            .strip_prefix(&repo)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let mut config = veil_config::Config::default();
        config.output.max_findings = Some(1);
        let rules = veil_core::get_all_rules(&config, vec![]);

        let aggregate =
            scan_paths_with_global_limit(vec![first_path, second_path], &rules, &config).unwrap();

        assert!(aggregate.limit_reached);
        assert_eq!(aggregate.findings.len(), 1);
        assert_eq!(aggregate.scanned_files, 1);
        assert!(aggregate
            .limit_reasons
            .contains(&"result-limit".to_string()));
        let _ = std::fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn fail_on_findings_zero_returns_error_envelope() {
        let state = Arc::new(AppState {
            token: "test-token".to_string(),
            run_cache: Arc::new(tokio::sync::RwLock::new(crate::evidence::RunCache::new(
                1, 1024, 1,
            ))),
            oauth: Arc::new(crate::auth::init_oauth()),
        });
        let request = ScanRequest {
            fail_on_findings: Some(0),
            ..ScanRequest::default()
        };

        let (status, Json(body)) = scan_project(State(state), Ok(Json(request)))
            .await
            .unwrap_err();

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(matches!(body.error.code, ErrorCode::InvalidRequest));
        assert_eq!(body.error.message, "failOnFindings must be >= 1");
    }

    #[tokio::test]
    async fn fail_on_score_above_range_returns_error_envelope() {
        let state = Arc::new(AppState {
            token: "test-token".to_string(),
            run_cache: Arc::new(tokio::sync::RwLock::new(crate::evidence::RunCache::new(
                1, 1024, 1,
            ))),
            oauth: Arc::new(crate::auth::init_oauth()),
        });
        let request = ScanRequest {
            fail_on_score: Some(101),
            ..ScanRequest::default()
        };

        let (status, Json(body)) = scan_project(State(state), Ok(Json(request)))
            .await
            .unwrap_err();

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(matches!(body.error.code, ErrorCode::InvalidRequest));
        assert_eq!(body.error.message, "failOnScore must be between 0 and 100");
    }

    #[tokio::test]
    async fn scan_json_rejection_returns_error_envelope() {
        let state = Arc::new(AppState {
            token: "test-token".to_string(),
            run_cache: Arc::new(tokio::sync::RwLock::new(crate::evidence::RunCache::new(
                1, 1024, 1,
            ))),
            oauth: Arc::new(crate::auth::init_oauth()),
        });
        let rejection =
            Json::<ScanRequest>::from_bytes(br#"{"failOnSeverity":"Severe"}"#).unwrap_err();

        let (status, Json(body)) = scan_project(State(state), Err(rejection))
            .await
            .unwrap_err();

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(matches!(body.error.code, ErrorCode::InvalidRequest));
        assert!(body.error.message.contains("Invalid JSON request body:"));
    }

    #[tokio::test]
    async fn baseline_json_rejection_returns_error_envelope() {
        let state = Arc::new(AppState {
            token: "test-token".to_string(),
            run_cache: Arc::new(tokio::sync::RwLock::new(crate::evidence::RunCache::new(
                1, 1024, 1,
            ))),
            oauth: Arc::new(crate::auth::init_oauth()),
        });
        let rejection =
            Json::<BaselineRequest>::from_bytes(br#"{"unexpectedField":true}"#).unwrap_err();

        let (status, Json(body)) = write_baseline(State(state), Err(rejection))
            .await
            .unwrap_err();

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(matches!(body.error.code, ErrorCode::InvalidRequest));
        assert!(body.error.message.contains("Invalid JSON request body:"));
    }

    #[tokio::test]
    async fn scan_baseline_file_denial_returns_path_denied() {
        let state = Arc::new(AppState {
            token: "test-token".to_string(),
            run_cache: Arc::new(tokio::sync::RwLock::new(crate::evidence::RunCache::new(
                1, 1024, 1,
            ))),
            oauth: Arc::new(crate::auth::init_oauth()),
        });
        let request = ScanRequest {
            baseline_file: Some("../veil.baseline.json".to_string()),
            ..ScanRequest::default()
        };

        let (status, Json(body)) = scan_project(State(state), Ok(Json(request)))
            .await
            .unwrap_err();

        assert_eq!(status, StatusCode::FORBIDDEN);
        assert!(matches!(body.error.code, ErrorCode::PathDenied));
        assert!(matches!(
            body.error.next_action,
            Some(NextAction::NarrowScope)
        ));
    }

    #[tokio::test]
    async fn scan_returns_run_too_large_when_evidence_cannot_be_cached() {
        let root = unique_target_dir("run-too-large");
        std::fs::write(root.join("sample.txt"), "hello\n").unwrap();
        let repo = repo_root();
        let path = root
            .strip_prefix(&repo)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let state = Arc::new(AppState {
            token: "test-token".to_string(),
            run_cache: Arc::new(tokio::sync::RwLock::new(crate::evidence::RunCache::new(
                1, 1, 1,
            ))),
            oauth: Arc::new(crate::auth::init_oauth()),
        });
        let request = ScanRequest {
            paths: Some(vec![path]),
            ..ScanRequest::default()
        };

        let (status, Json(body)) = scan_project(State(state), Ok(Json(request)))
            .await
            .unwrap_err();

        assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
        assert!(matches!(body.error.code, ErrorCode::RunTooLarge));
        assert!(matches!(
            body.error.next_action,
            Some(NextAction::NarrowScope)
        ));
        let _ = std::fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn write_baseline_rejects_denied_input_path() {
        let state = Arc::new(AppState {
            token: "test-token".to_string(),
            run_cache: Arc::new(tokio::sync::RwLock::new(crate::evidence::RunCache::new(
                1, 1024, 1,
            ))),
            oauth: Arc::new(crate::auth::init_oauth()),
        });
        let request = BaselineRequest {
            paths: Some(vec!["../secrets".to_string()]),
            ..BaselineRequest::default()
        };

        let (status, Json(body)) = write_baseline(State(state), Ok(Json(request)))
            .await
            .unwrap_err();

        assert_eq!(status, StatusCode::FORBIDDEN);
        assert!(matches!(body.error.code, ErrorCode::PathDenied));
        assert!(matches!(
            body.error.next_action,
            Some(NextAction::NarrowScope)
        ));
    }

    #[tokio::test]
    async fn write_baseline_rejects_incomplete_scan() {
        let root = unique_target_dir("baseline-incomplete");
        let oversized = root.join("large.txt");
        let output = root.join("veil.baseline.json");
        std::fs::write(
            &oversized,
            "A".repeat((veil_core::DEFAULT_MAX_FILE_SIZE_BYTES + 1) as usize),
        )
        .unwrap();
        let repo = repo_root();
        let path = root
            .strip_prefix(&repo)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let output_path = output
            .strip_prefix(&repo)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let state = Arc::new(AppState {
            token: "test-token".to_string(),
            run_cache: Arc::new(tokio::sync::RwLock::new(crate::evidence::RunCache::new(
                1, 1024, 1,
            ))),
            oauth: Arc::new(crate::auth::init_oauth()),
        });
        let request = BaselineRequest {
            paths: Some(vec![path]),
            output_path: Some(output_path),
        };

        let (status, Json(body)) = write_baseline(State(state), Ok(Json(request)))
            .await
            .unwrap_err();

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(matches!(body.error.code, ErrorCode::InvalidRequest));
        assert!(body
            .error
            .message
            .contains("Cannot write baseline from incomplete scan: max-file-size"));
        assert!(matches!(
            body.error.next_action,
            Some(NextAction::NarrowScope)
        ));
        assert!(!output.exists());
        let _ = std::fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn write_baseline_returns_error_when_save_fails() {
        let root = unique_target_dir("baseline-save-fails");
        let repo = repo_root();
        let path = root
            .strip_prefix(&repo)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let state = Arc::new(AppState {
            token: "test-token".to_string(),
            run_cache: Arc::new(tokio::sync::RwLock::new(crate::evidence::RunCache::new(
                1, 1024, 1,
            ))),
            oauth: Arc::new(crate::auth::init_oauth()),
        });
        let request = BaselineRequest {
            paths: Some(vec![path.clone()]),
            output_path: Some(path),
        };

        let (status, Json(body)) = write_baseline(State(state), Ok(Json(request)))
            .await
            .unwrap_err();

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(matches!(body.error.code, ErrorCode::InvalidRequest));
        assert!(body.error.message.contains("Failed to write baseline:"));
        assert!(matches!(
            body.error.next_action,
            Some(NextAction::NarrowScope)
        ));
        let _ = std::fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn doctor_reports_scanner_default_bounds() {
        let state = Arc::new(AppState {
            token: "test-token".to_string(),
            run_cache: Arc::new(tokio::sync::RwLock::new(crate::evidence::RunCache::new(
                1, 1024, 1,
            ))),
            oauth: Arc::new(crate::auth::init_oauth()),
        });

        let Json(body) = get_doctor(State(state)).await.unwrap();

        assert!(matches!(
            body.bounds.get("maxFileCount"),
            Some(BoundValue::Number(value)) if *value == veil_core::DEFAULT_MAX_FILE_COUNT as u64
        ));
        assert!(matches!(
            body.bounds.get("maxFileSizeBytes"),
            Some(BoundValue::Number(value)) if *value == veil_core::DEFAULT_MAX_FILE_SIZE_BYTES
        ));
    }

    #[test]
    fn local_api_logs_preset_without_rule_pack_returns_guidance() {
        let config =
            apply_request_preset_for_api(veil_config::Config::default(), Some(PresetName::LogsJp))
                .unwrap();
        let (status, Json(body)) =
            load_rules_for_scan_with_preset(&config, Some(PresetName::LogsJp)).unwrap_err();

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(matches!(body.error.code, ErrorCode::InvalidRequest));
        assert!(body
            .error
            .message
            .contains("Preset 'logs-jp' requires the log rule pack"));
        assert!(body.error.message.contains("veil init --preset logs-jp"));
    }

    #[tokio::test]
    async fn scan_rejects_unimplemented_staged_and_ci_modes() {
        for mode in [ScanMode::Staged, ScanMode::Ci] {
            let state = Arc::new(AppState {
                token: "test-token".to_string(),
                run_cache: Arc::new(tokio::sync::RwLock::new(crate::evidence::RunCache::new(
                    1, 1024, 1,
                ))),
                oauth: Arc::new(crate::auth::init_oauth()),
            });
            let request = ScanRequest {
                mode: Some(mode),
                ..ScanRequest::default()
            };

            let (status, Json(body)) = scan_project(State(state), Ok(Json(request)))
                .await
                .unwrap_err();

            assert_eq!(status, StatusCode::BAD_REQUEST);
            assert!(matches!(body.error.code, ErrorCode::InvalidRequest));
            assert_eq!(
                body.error.message,
                "mode staged/ci is not implemented by the Local API in PR-0; use mode full or omit mode."
            );
        }
    }
}

#[cfg(test)]
mod evidence_tests {
    #[tokio::test]
    async fn test_evidence_pack_contract() {
        use crate::AppState;
        use axum::extract::{Path, State};
        use std::sync::Arc;

        let token_val = "sensitive_token_leakage_test_123456789".to_string();
        let state = Arc::new(AppState {
            token: token_val.clone(),
            run_cache: Arc::new(tokio::sync::RwLock::new(crate::evidence::RunCache::new(
                5, 10_000_000, 5,
            ))),
            oauth: Arc::new(crate::auth::init_oauth()),
        });

        let empty_counts = crate::api::SeverityCounts::zero();
        let summary = crate::api::EvidenceSummary {
            total_findings: 0,
            suppressed_findings: 0,
            effective_findings: 0,
            severity_counts: empty_counts.clone(),
            all_severity_counts: empty_counts.clone(),
            suppressed_severity_counts: empty_counts,
            coverage_complete: true,
        };
        let (meta, cached_run) = crate::evidence_generator::generate_evidence_pack(
            &veil_config::Config::default(),
            &[],
            summary,
            crate::api::RunStatus::Success,
            false,
            Vec::new(),
            1,
            0,
            1,
            None,
        );
        let run_id = meta.run_id.clone();

        let inserted = state
            .run_cache
            .write()
            .await
            .insert(run_id.clone(), cached_run);
        assert!(inserted);

        let mut cache = state.run_cache.write().await;
        let cached = cache.get(&run_id).unwrap();

        assert!(
            cached.baseline_json.is_none(),
            "Baseline should be omitted by default"
        );
        let meta_json = serde_json::to_value(&cached.meta).unwrap();
        assert_eq!(meta_json["schemaVersion"], "veil-pro-run-meta-v1");

        let report_json: serde_json::Value = serde_json::from_str(&cached.report_json).unwrap();
        assert_eq!(report_json["schemaVersion"], "veil-evidence-report-v1");

        let forbidden_strings = vec![
            format!("?token={}", token_val),
            format!("#token={}", token_val),
            format!("Authorization: Bearer {}", token_val),
            token_val.clone(),
        ];

        let check_no_tokens = |content: &str, file: &str| {
            for token in &forbidden_strings {
                assert!(
                    !content.contains(token),
                    "Forbidden exact token fragment leaked into {}",
                    file
                );
            }
        };

        check_no_tokens(&cached.report_html, "report.html");
        check_no_tokens(&cached.report_json, "report.json");
        check_no_tokens(&cached.effective_config, "effective_config.toml");
        let meta_str = serde_json::to_string(&cached.meta).unwrap();
        check_no_tokens(&meta_str, "run_meta.json");

        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(cached.report_json.as_bytes());
        let sha = hex::encode(hasher.finalize());
        assert_eq!(
            cached.meta.artifacts.report_json.sha256, sha,
            "report.json SHA256 must match run_meta artifacts exactly"
        );

        drop(cache);
        let _ = crate::api::get_run_meta(State(state.clone()), Path(run_id)).await;
    }
}
