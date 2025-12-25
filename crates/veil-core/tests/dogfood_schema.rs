use std::env;
use std::fs;
use std::path::PathBuf;
use veil_core::metrics::reason::MetricsV1;

/// Validates that all generated dogfood logs strictly adhere to the Rust contracts (and thus the Schema).
#[test]
fn test_validate_dogfood_schema_compliance() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let root_dir = PathBuf::from(manifest_dir).parent().unwrap().parent().unwrap().to_path_buf();
    let dogfood_dir = root_dir.join("docs").join("dogfood");

    if !dogfood_dir.exists() {
        // May not exist in fresh clones or CI if not generated yet.
        // But for this phase, we assume it exists or we skip.
        println!("Dogfood dir not found, skipping validation: {:?}", dogfood_dir);
        return;
    }

    let mut count = 0;
    for entry in fs::read_dir(&dogfood_dir).expect("Failed to read dogfood dir") {
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();
        if path.is_dir() {
            let metrics_path = path.join("metrics_v1.json");
            if metrics_path.exists() {
                validate_metrics_file(&metrics_path);
                count += 1;
            }
        }
    }
    println!("Validated {} dogfood entries.", count);
}

fn validate_metrics_file(path: &PathBuf) {
    let content = fs::read_to_string(path).unwrap_or_else(|_| panic!("Failed to read {:?}", path));
    
    // Strict deserialization (due to deny_unknown_fields in struct)
    match serde_json::from_str::<MetricsV1>(&content) {
        Ok(m) => {
            assert_eq!(m.v, 1, "Version mismatch in {:?}", path);
            // Additional checks if needed
        }
        Err(e) => {
            panic!("Schema contract violation in {:?}: {}", path, e);
        }
    }
}
