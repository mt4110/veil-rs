use std::fs;
use std::path::PathBuf;
use veil_guardian::providers::osv::client::BatchResponse;

#[test]
fn test_contract_osv_fixtures() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let fixtures_dir = PathBuf::from(manifest_dir).join("../../tests/fixtures/osv");

    assert!(
        fixtures_dir.exists(),
        "Fixtures directory not found at {:?}",
        fixtures_dir
    );

    for entry in fs::read_dir(fixtures_dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            println!("Testing contract for: {:?}", path.file_name().unwrap());
            let content = fs::read_to_string(&path).unwrap();

            // Deserializing into BatchResponse checks if the JSON structure
            // is compatible with what the application expects.
            match serde_json::from_str::<BatchResponse>(&content) {
                Ok(_) => {
                    // Success
                }
                Err(e) => {
                    panic!(
                        "Contract Violation: Fixture {:?} cannot be deserialized into BatchResponse. Error: {}",
                        path.file_name().unwrap(),
                        e
                    );
                }
            }
        }
    }
}
