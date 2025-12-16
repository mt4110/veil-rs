use std::path::PathBuf;

use veil_guardian::{scan_lockfile, ScanOptions};
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

#[test]
fn test_npm_integration() {
    // 1. Setup Mock Server for OSV using a manual runtime
    // We use a separate runtime because `reqwest::blocking` cannot be easily used
    // inside a tokio::test (which sets up a runtime on the current thread).
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mock_server = rt.block_on(wiremock::MockServer::start());

    // Mock response for querybatch
    // We expect a query for lodash 4.17.15
    // Response should be a BatchResponse with 1 result containing vulns
    let body = r#"{
        "results": [
            {
                "vulns": [
                    {
                        "id": "GHSA-p6mc-m468-83gw",
                        "summary": "Prototype Pollution in lodash",
                        "details": "Lodash versions prior to 4.17.19 are vulnerable to Prototype Pollution."
                    }
                ]
            }
        ]
    }"#;

    rt.block_on(async {
        Mock::given(method("POST"))
            .and(path("/v1/querybatch"))
            .respond_with(ResponseTemplate::new(200).set_body_string(body))
            .mount(&mock_server)
            .await;
    });

    // Configure URL directly for injection
    let osv_url = mock_server.uri() + "/v1/querybatch";

    // 2. Setup Temp File
    // We need the file to be named "package-lock.json" for the scanner to recognize it.
    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
    let lock_path = temp_dir.path().join("package-lock.json");

    // Read fixture content
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/vulnerable.package-lock.json");
    std::fs::copy(fixture_path, &lock_path).expect("failed to copy fixture");

    // 3. Scan
    let result = scan_lockfile(
        &lock_path,
        ScanOptions {
            offline: false,
            show_details: false,
            osv_api_url: Some(osv_url),
            metrics: None,
        },
    )
    .expect("Scan failed");

    // 4. Verify results
    assert_eq!(result.scanned_crates, 1); // Only lodash (root skipped)
    assert_eq!(result.vulnerabilities.len(), 1);
    assert_eq!(result.vulnerabilities[0].package_name, "lodash");
    assert_eq!(result.vulnerabilities[0].version, "4.17.15");
    assert_eq!(
        result.vulnerabilities[0].advisories[0].id,
        "GHSA-p6mc-m468-83gw"
    );
}
