use anyhow::{Context, Result};
use clap::Parser;
use schemars::schema_for;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use utoipa::openapi::security::{HttpAuthScheme, HttpBuilder, SecurityScheme};
use utoipa::{Modify, OpenApi};

#[allow(dead_code)]
#[path = "../api/dto.rs"]
mod dto;
use dto::*;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value = "schemas")]
    out_dir: PathBuf,
}

#[allow(dead_code)]
#[utoipa::path(
    get,
    path = "/api/me",
    responses(
        (status = 200, description = "Auth context", body = AuthContext),
        (status = 401, description = "Unauthorized", body = ErrorEnvelope)
    ),
    security(("bearerAuth" = []))
)]
fn openapi_get_me() {}

#[allow(dead_code)]
#[utoipa::path(
    get,
    path = "/api/projects",
    responses((status = 200, description = "Project list", body = ProjectsResponse)),
    security(("bearerAuth" = []))
)]
fn openapi_list_projects() {}

#[allow(dead_code)]
#[utoipa::path(
    post,
    path = "/api/scan",
    request_body = ScanRequest,
    responses(
        (status = 200, description = "Scan response", body = ScanResponse),
        (status = 400, description = "Invalid request", body = ErrorEnvelope),
        (status = 403, description = "Path denied", body = ErrorEnvelope),
        (status = 413, description = "Run too large", body = ErrorEnvelope)
    ),
    security(("bearerAuth" = []))
)]
fn openapi_scan_project() {}

#[allow(dead_code)]
#[utoipa::path(
    get,
    path = "/api/runs/{runId}",
    params(("runId" = String, Path, description = "Run identifier")),
    responses(
        (status = 200, description = "Run metadata", body = RunMetaResponse),
        (status = 410, description = "Run expired", body = ErrorEnvelope)
    ),
    security(("bearerAuth" = []))
)]
fn openapi_get_run_meta() {}

#[allow(dead_code)]
#[utoipa::path(
    get,
    path = "/api/runs/{runId}/evidence.zip",
    params(("runId" = String, Path, description = "Run identifier")),
    responses(
        (status = 200, description = "Evidence ZIP", content_type = "application/zip"),
        (status = 410, description = "Run expired", body = ErrorEnvelope),
        (status = 500, description = "Evidence assembly failed", body = ErrorEnvelope)
    ),
    security(("bearerAuth" = []))
)]
fn openapi_export_evidence() {}

#[allow(dead_code)]
#[utoipa::path(
    get,
    path = "/api/policy",
    responses((status = 200, description = "Effective policy", body = PolicyResponse)),
    security(("bearerAuth" = []))
)]
fn openapi_get_policy() {}

#[allow(dead_code)]
#[utoipa::path(
    post,
    path = "/api/baseline",
    request_body = BaselineRequest,
    responses(
        (status = 200, description = "Baseline result", body = BaselineResponse),
        (status = 403, description = "Path denied", body = ErrorEnvelope)
    ),
    security(("bearerAuth" = []))
)]
fn openapi_write_baseline() {}

#[allow(dead_code)]
#[utoipa::path(
    get,
    path = "/api/doctor",
    responses((status = 200, description = "Doctor diagnostics", body = DoctorResponse)),
    security(("bearerAuth" = []))
)]
fn openapi_get_doctor() {}

#[derive(utoipa::OpenApi)]
#[openapi(
    paths(
        openapi_get_me,
        openapi_list_projects,
        openapi_scan_project,
        openapi_get_run_meta,
        openapi_export_evidence,
        openapi_get_policy,
        openapi_write_baseline,
        openapi_get_doctor
    ),
    components(schemas(
        ArtifactMeta,
        AuthContext,
        BaselineArtifactMeta,
        BaselineRequest,
        BaselineResponse,
        DoctorResponse,
        EngineMeta,
        ErrorEnvelope,
        EvidenceArtifacts,
        EvidenceReportV1,
        EvidenceSummary,
        FindingV1,
        PolicyResponse,
        PrivacyMeta,
        ProductMeta,
        ProjectsResponse,
        ProjectSummary,
        RulePackMeta,
        RunMetaResponse,
        RunResultMeta,
        SafeFindingApiV1,
        ScanRequest,
        ScanResponse,
        SeverityCounts
    )),
    modifiers(&SecurityAddon)
)]
struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        let components = openapi.components.get_or_insert_with(Default::default);
        components.add_security_scheme(
            "bearerAuth",
            SecurityScheme::Http(
                HttpBuilder::new()
                    .scheme(HttpAuthScheme::Bearer)
                    .bearer_format("opaque")
                    .build(),
            ),
        );
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    fs::create_dir_all(&args.out_dir).with_context(|| {
        format!(
            "failed to create schema output directory {}",
            args.out_dir.display()
        )
    })?;

    write_json_schema::<SafeFindingApiV1>(
        &args.out_dir,
        "json-schema.safe-finding-api.json",
        "https://veil.local/schemas/safe-finding-api-v1.json",
        "SafeFindingApiV1",
        Some("Local UI / Evidence preview finding schema. Raw line content and matched content are forbidden. Field names are camelCase."),
    )?;
    write_json_schema::<EvidenceReportV1>(
        &args.out_dir,
        "json-schema.report.json",
        "https://veil.local/schemas/evidence-report-v1.json",
        "Veil Evidence Report v1",
        None,
    )?;
    write_json_schema::<RunMetaV1>(
        &args.out_dir,
        "json-schema.run-meta.json",
        "https://veil.local/schemas/run-meta-v1.json",
        "Veil Pro RunMeta v1",
        None,
    )?;
    write_json_schema::<FindingV1>(
        &args.out_dir,
        "json-schema.finding.json",
        "https://veil.local/schemas/finding-v1.json",
        "FindingV1 (CLI/Core snake_case)",
        Some("CLI/Core compatible finding schema. Local UI must use SafeFindingApiV1 instead."),
    )?;
    write_openapi(&args.out_dir)?;

    Ok(())
}

fn write_json_schema<T>(
    out_dir: &Path,
    file_name: &str,
    id: &str,
    title: &str,
    description: Option<&str>,
) -> Result<()>
where
    T: schemars::JsonSchema,
{
    let mut schema = serde_json::to_value(schema_for!(T))?;
    normalize_json_schema(&mut schema, id, title, description);
    write_json_pretty(&out_dir.join(file_name), &schema)
}

fn normalize_json_schema(schema: &mut Value, id: &str, title: &str, description: Option<&str>) {
    if let Some(object) = schema.as_object_mut() {
        object.insert(
            "$schema".to_string(),
            Value::String("https://json-schema.org/draft/2020-12/schema".to_string()),
        );
        object.insert("$id".to_string(), Value::String(id.to_string()));
        object.insert("title".to_string(), Value::String(title.to_string()));
        if let Some(description) = description {
            object.insert(
                "description".to_string(),
                Value::String(description.to_string()),
            );
        }
    }
    apply_contract_schema_fixes(schema);
    sort_json(schema);
}

fn apply_contract_schema_fixes(value: &mut Value) {
    match value {
        Value::Object(map) => {
            if let Some(Value::Object(properties)) = map.get_mut("properties") {
                if properties.contains_key("result") && properties.contains_key("artifacts") {
                    set_const(properties, "schemaVersion", "veil-pro-run-meta-v1");
                }
                if properties.contains_key("summary") && properties.contains_key("findings") {
                    set_const(properties, "schemaVersion", "veil-evidence-report-v1");
                }
                if let Some(path_schema) = properties.get_mut("path") {
                    if path_schema
                        .get("enum")
                        .and_then(Value::as_array)
                        .map(|values| values.iter().any(|v| v == "veil.baseline.json"))
                        .unwrap_or(false)
                    {
                        *path_schema = json!({ "const": "veil.baseline.json" });
                    }
                }
            }
            for child in map.values_mut() {
                apply_contract_schema_fixes(child);
            }
        }
        Value::Array(values) => {
            for child in values {
                apply_contract_schema_fixes(child);
            }
        }
        _ => {}
    }
}

fn set_const(properties: &mut serde_json::Map<String, Value>, name: &str, expected: &str) {
    if let Some(schema) = properties.get_mut(name) {
        *schema = json!({ "const": expected });
    }
}

fn write_openapi(out_dir: &Path) -> Result<()> {
    let mut openapi = serde_json::to_value(ApiDoc::openapi())?;
    if let Some(object) = openapi.as_object_mut() {
        object.insert("openapi".to_string(), Value::String("3.0.3".to_string()));
        object.insert(
            "info".to_string(),
            json!({
                "title": "Veil Pro Local API",
                "version": "1.0.0-local",
                "description": "Generated from Rust DTOs. Do not hand-edit in implementation."
            }),
        );
        object.insert(
            "servers".to_string(),
            json!([{
                "url": "http://127.0.0.1:{port}",
                "variables": {
                    "port": { "default": "3000" }
                }
            }]),
        );
    }
    sort_json(&mut openapi);
    let yaml = serde_yaml::to_string(&openapi)?;
    fs::write(out_dir.join("openapi.local-api.yaml"), yaml)?;
    Ok(())
}

fn write_json_pretty(path: &Path, value: &Value) -> Result<()> {
    let text = serde_json::to_string_pretty(value)?;
    fs::write(path, format!("{text}\n"))?;
    Ok(())
}

fn sort_json(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for child in map.values_mut() {
                sort_json(child);
            }
            let sorted = map
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect::<BTreeMap<_, _>>();
            map.clear();
            map.extend(sorted);
        }
        Value::Array(values) => {
            for child in values {
                sort_json(child);
            }
        }
        _ => {}
    }
}
