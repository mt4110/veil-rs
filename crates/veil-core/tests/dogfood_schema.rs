use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use valico::json_schema;
use veil_core::metrics::reason::{MetricsBody, MetricsV1};

fn validate_json(schema_path: &str, instance: &Value) {
    let schema_text = fs::read_to_string(schema_path)
        .expect(&format!("Failed to read schema file: {}", schema_path));

    let schema_json: Value =
        serde_json::from_str(&schema_text).expect("Failed to parse schema JSON");

    let mut scope = json_schema::Scope::new();
    let schema = scope
        .compile_and_return(schema_json, false)
        .expect("Failed to compile schema");

    let state = schema.validate(instance);
    if !state.is_valid() {
        panic!(
            "Schema validation failed for {}: {:?}",
            schema_path, state.errors
        );
    }
}

#[test]
fn test_metrics_schema_validity() {
    // 1. Generate a valid MetricsV1 object
    let mut counts = BTreeMap::new();
    counts.insert("config_invalid".to_string(), 10);
    counts.insert("offline".to_string(), 5);

    let metrics = MetricsV1 {
        v: 1,
        metrics: MetricsBody {
            counts_by_reason: counts,
            counts_by_hint: BTreeMap::new(),
        },
        meta: Some(serde_json::json!({
            "generated_by": "test",
            "week_id": "2025-W52-Tokyo"
        })),
    };

    let metrics_json = serde_json::to_value(&metrics).unwrap();

    // 2. Locate Schema
    let root = env!("CARGO_MANIFEST_DIR");
    let schema_path = PathBuf::from(root)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("schemas/metrics_v1.schema.json");

    // 3. Validate
    validate_json(schema_path.to_str().unwrap(), &metrics_json);
}

#[test]
fn test_worklist_schema_validity() {
    // 1. Generate a mock Worklist V1 object (manually constructed as struct isn't in Rust yet)
    let worklist_json = serde_json::json!({
        "v": 1,
        "week_id": "2025-W52-Tokyo",
        "generated_at": "2025-12-25T12:00:00Z",
        "git_sha": "abc1234",
        "items": [
            {
                "rank": 1,
                "action_id": "A-NET-001",
                "title": "Fix Network",
                "score": 100,
                "signals": {
                    "count_now": 50,
                    "delta": 10,
                    "hint_key": "check_network"
                },
                "suggested_paths": ["/etc/hosts"]
            }
        ]
    });

    // 2. Locate Schema
    let root = env!("CARGO_MANIFEST_DIR");
    let schema_path = PathBuf::from(root)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("schemas/worklist_v1.schema.json");

    // 3. Validate
    validate_json(schema_path.to_str().unwrap(), &worklist_json);
}
