use std::path::PathBuf;

use serde_json::json;
use veil_guardian::{scan_lockfile, ScanOptions};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[test]
fn test_yarn_integration() {
    // 1. Setup WireMock for OSV API
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mock_server = rt.block_on(MockServer::start());

    // Use the Classic yarn fixture which contains lodash@4.17.21
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();

    // Mock OSV response for lodash
    let response_body = json!({
        "results": [
            { "vulns": [] }, // @types/node

            { "vulns": [] }, // fsevents
            {
                "vulns": [
                    {
                        "id": "GHSA-35jh-r3h4-6jhm",
                        "summary": "Command Injection in lodash",
                        "details": "The lodash package is vulnerable to Command Injection...",
                        "affected": [
                            {
                                "package": {
                                    "name": "lodash",
                                    "ecosystem": "npm",
                                    "purl": "pkg:npm/lodash"
                                },
                                "ranges": [
                                    {
                                        "type": "SEMVER",
                                        "events": [
                                            {"introduced": "0.0.0"},
                                            {"fixed": "4.17.22"}
                                        ]
                                    }
                                ]
                            }
                        ]
                    }
                ]
            }
        ]
    });

    rt.block_on(async {
        Mock::given(method("POST"))
            .and(path("/querybatch"))
            .respond_with(ResponseTemplate::new(200).set_body_json(response_body))
            .mount(&mock_server)
            .await;
    });

    // 2. Setup Temp File (Copy fixture to yarn.lock)
    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
    let lockfile_path = temp_dir.path().join("yarn.lock");

    let fixture_path =
        PathBuf::from(manifest_dir).join("../../tests/fixtures/yarn/classic.yarn.lock");
    std::fs::copy(fixture_path, &lockfile_path).expect("failed to copy fixture");

    // 3. Set environment variable to redirect OSV traffic
    // 3. Set environment variable to redirect OSV traffic

    // 4. Run the scanner
    let options = ScanOptions {
        offline: false,
        show_details: false,
        osv_api_url: Some(mock_server.uri() + "/v1/querybatch"),
        metrics: None,
        cache_dir: None,
    };
    let result = scan_lockfile(&lockfile_path, options).unwrap();

    // 5. Verify results
    assert_eq!(result.scanned_crates, 3);

    // Check vulnerabilities
    // We expect one vuln for lodash
    let vuln = result
        .vulnerabilities
        .iter()
        .find(|v| v.package_name == "lodash")
        .expect("Should find vulnerability for lodash");

    assert_eq!(vuln.version, "4.17.21");
    assert!(!vuln.advisories.is_empty());
    assert!(vuln
        .advisories
        .iter()
        .any(|a| a.id == "GHSA-35jh-r3h4-6jhm"));
}
