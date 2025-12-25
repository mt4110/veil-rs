use serde_json::{Map, Value};
use std::env;
use std::fs;
use std::path::PathBuf;
use veil_core::metrics::reason::{MetricsV1, ReasonCode};

/// Canonicalize JSON value for deterministic comparison.
/// - Recursively sorts object keys.
/// - Arrays preserve order (as per spec).
fn canonicalize_json(v: &Value) -> String {
    let sorted_v = sort_keys(v);
    // serde_json::to_string serializes Map in insertion order (which is now sorted)
    serde_json::to_string(&sorted_v).expect("Canonical serialization failed")
}

/// Recursively sort object keys
fn sort_keys(v: &Value) -> Value {
    match v {
        Value::Object(map) => {
            let mut sorted_map = Map::new();
            // Collect keys to sort them
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();

            for k in keys {
                let val = map.get(k).unwrap();
                sorted_map.insert(k.clone(), sort_keys(val));
            }
            Value::Object(sorted_map)
        }
        Value::Array(arr) => {
            Value::Array(arr.iter().map(sort_keys).collect())
        }
        _ => v.clone(),
    }
}

#[test]
fn test_metrics_v1_golden() {
    // 1. Setup Data
    let mut m = MetricsV1::new();
    
    // Fill with some data to verify sorting and snake_case
    // We insert in random-ish order to prove output is sorted by key
    // Note: MetricsV1.counts_by_reason uses BTreeMap, so it is ALREADY sorted.
    // This test primarily verifies that the JSON serialization respects that,
    // and that any future fields (or 'meta' if present) are also handled.
    m.metrics.counts_by_reason.insert(ReasonCode::IoReadFailed.as_str().to_string(), 10);
    m.metrics.counts_by_reason.insert(ReasonCode::ConfigInvalid.as_str().to_string(), 5);
    m.metrics.counts_by_reason.insert(ReasonCode::Unexpected.as_str().to_string(), 1);
    
    // 2. Serialize to Value
    let v_raw = serde_json::to_value(&m).expect("Failed to serialize struct");
    
    // 3. Canonicalize (Force Sort Keys - although likely already sorted for this struct)
    let output_json = canonicalize_json(&v_raw);

    // 4. Golden Path
    let mut golden_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    golden_path.push("tests/golden/metrics_v1.canonical.json");

    // 5. Update or Verify
    if env::var("UPDATE_GOLDEN").unwrap_or_default() == "1" {
        fs::create_dir_all(golden_path.parent().unwrap()).ok();
        fs::write(&golden_path, &output_json).expect("Failed to update golden file");
        println!("Updated golden file: {:?}", golden_path);
    } else {
        if !golden_path.exists() {
            panic!("Golden file missing: {:?}\nRun `UPDATE_GOLDEN=1 cargo test -p veil-core --test golden_metrics` to generate it.", golden_path);
        }
        let expected = fs::read_to_string(&golden_path).expect("Failed to read golden file");
        // Trim newline if editor added one
        let expected = expected.trim();
        
        assert_eq!(output_json, expected, "Golden JSON mismatch!\nLeft: {}\nRight: {}\nRun `UPDATE_GOLDEN=1 cargo test -p veil-core --test golden_metrics` to update.", output_json, expected);
    }
}
