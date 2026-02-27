use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tower_sessions::Session;
use veil_core::Severity;

// GET /api/me
// Returns the currently authenticated context (either SSO user or local token context)
pub async fn get_me(session: Session) -> Result<Json<serde_json::Value>, StatusCode> {
    if let Ok(Some(email)) = session.get::<String>("user_email").await {
        return Ok(Json(serde_json::json!({
            "authenticated": true,
            "type": "sso",
            "email": email,
            "name": session.get::<String>("user_name").await.unwrap_or_default().unwrap_or_default()
        })));
    }

    // If we're here and the request passed middleware, it's a CLI Token user
    Ok(Json(serde_json::json!({
        "authenticated": true,
        "type": "local_token"
    })))
}

use crate::AppState;

// --- DTOs ---

#[derive(Serialize)]
pub struct ApiError {
    pub error: String,
}

#[derive(Deserialize)]
pub struct ScanRequest {
    pub paths: Vec<String>,
}

#[derive(Serialize)]
pub struct SafeFinding {
    pub path: String,
    pub line_number: usize,
    pub rule_id: String,
    pub severity: Severity,
    // We MUST NOT expose raw matched_content or line_content to UI
    pub masked_snippet: String,
}

#[derive(Serialize)]
pub struct ScanResponse {
    pub run_id: String,
    pub scanned_files: usize,
    pub skipped_files: usize,
    pub findings: Vec<SafeFinding>,
}

#[derive(Serialize)]
pub struct ProjectsResponse {
    pub current_dir: String,
}

#[derive(Serialize)]
pub struct PolicyResponse {
    pub has_org_config: bool,
    pub org_config_path: Option<String>,
    pub effective_rules_count: usize,
}

// --- Endpoints ---

pub async fn list_projects(State(_state): State<Arc<AppState>>) -> Json<ProjectsResponse> {
    let current_dir = std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| ".".to_string());

    Json(ProjectsResponse { current_dir })
}

pub async fn scan_project(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ScanRequest>,
) -> Json<ScanResponse> {
    let mut config = veil_config::Config::default();

    // Load config from the first path (project root)
    if let Some(first_path) = req.paths.first() {
        if let Ok(layers) =
            crate::config_loader::load_config_layers(Some(&PathBuf::from(first_path)))
        {
            config = layers.effective;
        }
    } else {
        if let Ok(layers) = crate::config_loader::load_config_layers(None) {
            config = layers.effective;
        }
    }

    // For web UI, we definitely want some masking unless requested otherwise
    let rules = veil_core::get_all_rules(&config, vec![]);

    let mut safe_findings = Vec::new();
    let mut scanned_files = 0;
    let mut skipped_files = 0;

    let paths_to_scan = if req.paths.is_empty() {
        vec![".".to_string()]
    } else {
        req.paths
    };

    for p in paths_to_scan {
        match validate_safe_path(&p) {
            Ok(safe_path) => {
                let result = veil_core::scan_path(&safe_path, &rules, &config);
                scanned_files += result.scanned_files;
                skipped_files += result.skipped_files;

                for f in result.findings {
                    safe_findings.push(SafeFinding {
                        path: f.path.to_string_lossy().to_string(),
                        line_number: f.line_number,
                        rule_id: f.rule_id,
                        severity: f.severity,
                        masked_snippet: f.masked_snippet, // strictly mapped
                    });
                }
            }
            Err(e) => {
                // Log and skip unsafe path
                eprintln!("Blocked unsafe scan path: {}", e);
            }
        }
    }

    // Attempt to load baseline if it exists to bundle it
    let mut has_baseline = false;
    let mut baseline_content = None;
    if let Ok(b) = std::fs::read_to_string(".veil-baseline.json") {
        has_baseline = true;
        baseline_content = Some(b);
    }

    let (run_meta, cached_run) = crate::evidence_generator::generate_evidence_pack(
        &config,
        &safe_findings,
        scanned_files,
        skipped_files,
        150, // duration mock
        has_baseline,
        baseline_content,
    );

    state
        .run_cache
        .write()
        .await
        .insert(run_meta.run_id.clone(), cached_run);

    Json(ScanResponse {
        run_id: run_meta.run_id,
        scanned_files,
        skipped_files,
        findings: safe_findings,
    })
}

pub async fn get_policy(State(_state): State<Arc<AppState>>) -> Json<PolicyResponse> {
    let layers = crate::config_loader::load_config_layers(None).unwrap_or_default();
    let config = layers.effective;
    let rules = veil_core::get_all_rules(&config, vec![]);

    let has_org_config = layers.org.is_some();
    // Simulate finding the org config Path (since config_loader abstracts it, we do a basic check)
    let org_config_path = std::env::var("VEIL_ORG_CONFIG").ok();

    Json(PolicyResponse {
        has_org_config,
        org_config_path,
        effective_rules_count: rules.len(),
    })
}

// --- Baseline Endpoint ---

#[derive(Deserialize)]
pub struct BaselineRequest {
    pub paths: Vec<String>,
}

#[derive(Serialize)]
pub struct BaselineResponse {
    pub success: bool,
    pub message: String,
    pub findings_count: usize,
    pub file_path: String,
}

pub async fn write_baseline(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<BaselineRequest>,
) -> Json<BaselineResponse> {
    let mut config = veil_config::Config::default();
    if let Some(first_path) = req.paths.first() {
        if let Ok(layers) =
            crate::config_loader::load_config_layers(Some(&PathBuf::from(first_path)))
        {
            config = layers.effective;
        }
    } else {
        if let Ok(layers) = crate::config_loader::load_config_layers(None) {
            config = layers.effective;
        }
    }

    let rules = veil_core::get_all_rules(&config, vec![]);
    let mut all_findings = Vec::new();

    let paths_to_scan = if req.paths.is_empty() {
        vec![".".to_string()]
    } else {
        req.paths
    };

    for p in paths_to_scan {
        if let Ok(safe_path) = validate_safe_path(&p) {
            let result = veil_core::scan_path(&safe_path, &rules, &config);
            all_findings.extend(result.findings);
        }
    }

    let val_version = env!("CARGO_PKG_VERSION");
    let snapshot = veil_core::baseline::from_findings(&all_findings, val_version);

    let mut baseline_path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    baseline_path.push(".veil-baseline.json");

    match veil_core::baseline::save_baseline(&baseline_path, &snapshot) {
        Ok(_) => Json(BaselineResponse {
            success: true,
            message: "Baseline created successfully.".to_string(),
            findings_count: snapshot.entries.len(),
            file_path: baseline_path.to_string_lossy().to_string(),
        }),
        Err(e) => Json(BaselineResponse {
            success: false,
            message: format!("Failed to save baseline: {}", e),
            findings_count: 0,
            file_path: "".to_string(),
        }),
    }
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

    // Check canonicalized prefix to avoid symlink trickery if the path or parent exists
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_safe_path() {
        // Valid paths
        assert!(validate_safe_path(".").is_ok());
        assert!(validate_safe_path("./src").is_ok());

        // Invalid paths (traversal)
        assert!(validate_safe_path("..").is_err());
        assert!(validate_safe_path("../some_other_folder").is_err());
        assert!(validate_safe_path("src/../..").is_err());

        // Invalid paths (absolute outside of cwd)
        #[cfg(unix)]
        assert!(validate_safe_path("/etc/passwd").is_err());
        #[cfg(windows)]
        assert!(validate_safe_path("C:\\Windows\\System32").is_err());
    }
}

pub async fn get_run_meta(
    State(state): State<Arc<AppState>>,
    Path(run_id): Path<String>,
) -> Result<Json<crate::evidence::RunMeta>, (StatusCode, Json<ApiError>)> {
    let mut cache = state.run_cache.write().await;
    if let Some(cached) = cache.get(&run_id) {
        Ok(Json(cached.meta))
    } else {
        Err((
            StatusCode::GONE,
            Json(ApiError {
                error: "Run evidence has expired or runId is invalid. Please trigger a new scan."
                    .to_string(),
            }),
        ))
    }
}

pub async fn export_evidence(
    State(state): State<Arc<AppState>>,
    Path(run_id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ApiError>)> {
    let mut cache = state.run_cache.write().await;
    let cached =
        match cache.get(&run_id) {
            Some(c) => c,
            None => return Err((
                StatusCode::GONE,
                Json(ApiError {
                    error:
                        "Run evidence has expired or runId is invalid. Please trigger a new scan."
                            .to_string(),
                }),
            )),
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
            let _ = zip.start_file("baseline.json", options);
            let _ = std::io::Write::write_all(&mut zip, baseline.as_bytes());
        }

        let _ = zip.finish().map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiError {
                    error: "Failed to assemble Evidence Pack ZIP archive.".to_string(),
                }),
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
mod evidence_tests {
    #[tokio::test]
    async fn test_evidence_pack_contract() {
        use crate::api::ScanRequest;
        use crate::AppState;
        use axum::extract::{Path, State};
        use axum::Json;
        use std::sync::Arc;

        let token_val = "sensitive_token_leakage_test_123456789".to_string();
        let state = Arc::new(AppState {
            token: token_val.clone(),
            run_cache: Arc::new(tokio::sync::RwLock::new(crate::evidence::RunCache::new(
                5, 10_000_000, 5,
            ))),
            oauth: Arc::new(crate::auth::init_oauth()),
        });

        let req = Json(ScanRequest {
            paths: vec!["/tmp/test_dir_veil.txt".to_string()],
        });
        let scan_res = crate::api::scan_project(State(state.clone()), req).await;
        let run_id = scan_res.0.run_id.clone();

        // Check if report builds
        let mut cache = state.run_cache.write().await;
        let cached = cache.get(&run_id).unwrap();

        assert!(
            cached.baseline_json.is_none(),
            "Baseline should be omitted by default"
        );
        assert_eq!(cached.meta.schema_version, "veil-pro-run-meta-v1");

        let report_json: serde_json::Value = serde_json::from_str(&cached.report_json).unwrap();
        assert_eq!(report_json["schemaVersion"], "veil-v1");

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
                    "Forbidden exact token fragment leaked into {}!",
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
    }
}
