use std::path::PathBuf;

use veil_guardian::{scan_lockfile, ScanOptions};
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

#[test]
fn test_pnpm_integration() {
    // 1. Setup Mock Server
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mock_server = rt.block_on(wiremock::MockServer::start());

    // Expecting 3 packages: @types/node, jest-config, lodash
    // Alphabetical order:
    // 1. @types/node (clean)
    // 2. jest-config (clean)
    // 3. lodash (vulnerable)

    let body = r#"{
        "results": [
            { "vulns": [] },
            { "vulns": [] },
            {
                "vulns": [
                    {
                        "id": "GHSA-p6mc-m468-83gw",
                        "summary": "Prototype Pollution in lodash",
                        "details": "Vulnerable."
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
    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
    let lock_path = temp_dir.path().join("pnpm-lock.yaml");

    // Read fixture content (v9)
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/pnpm/vulnerable.pnpm-lock.v9.yaml");
    std::fs::copy(fixture_path, &lock_path).expect("failed to copy fixture");

    // 3. Scan
    let result = scan_lockfile(
        &lock_path,
        ScanOptions {
            offline: false,
            show_details: false,
            osv_api_url: Some(osv_url),
            metrics: None,
            cache_dir: None,
        },
    )
    .expect("Scan failed");

    // 4. Verify results
    assert_eq!(result.scanned_crates, 3);
    assert_eq!(result.vulnerabilities.len(), 1);
    assert_eq!(result.vulnerabilities[0].package_name, "lodash");
    assert_eq!(result.vulnerabilities[0].version, "4.17.15");
}
