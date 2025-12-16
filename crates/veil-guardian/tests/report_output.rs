use insta::assert_snapshot;
use std::path::PathBuf;
use veil_guardian::{report::OutputFormat, scan_lockfile, ScanOptions};
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

#[test]
fn test_report_output_deterministic() {
    // 1. Setup Mock Server
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mock_server = rt.block_on(wiremock::MockServer::start());

    // Mock response for a-lib and b-lib (which are sorted by npm parser)
    let body = r#"{
        "results": [
            {
                "vulns": [
                    {
                        "id": "GHSA-1",
                        "summary": "Critical issue in A",
                        "details": "Details about GHSA-1",
                        "database_specific": { "severity": "CRITICAL" },
                        "affected": [ { "ranges": [ { "events": [ { "fixed": "1.0.1" } ] } ] } ]
                    },
                    {
                        "id": "GHSA-2",
                        "summary": "Medium issue in A",
                        "details": "Details about GHSA-2",
                        "database_specific": { "severity": "MODERATE" }
                    }
                ]
            },
            {
                "vulns": [
                    {
                        "id": "GHSA-3",
                        "summary": "Low issue in B",
                        "details": "Details about GHSA-3",
                        "database_specific": { "severity": "LOW" }
                    }
                ]
            }
        ]
    }"#;

    rt.block_on(async {
        // Query Batch Mock
        Mock::given(method("POST"))
            .and(path("/v1/querybatch"))
            .respond_with(ResponseTemplate::new(200).set_body_string(body))
            .mount(&mock_server)
            .await;

        // Custom Detail Mocks
        let ghsa1 = r#"{
            "id": "GHSA-1",
            "summary": "Critical issue in A",
            "details": "Details about GHSA-1",
            "affected": [ { "ranges": [ { "events": [ { "fixed": "1.0.1" } ] } ] } ],
            "database_specific": { "severity": "CRITICAL" }
        }"#;
        Mock::given(method("GET"))
            .and(path("/v1/vulns/GHSA-1"))
            .respond_with(ResponseTemplate::new(200).set_body_string(ghsa1))
            .mount(&mock_server)
            .await;

        let ghsa2 = r#"{
            "id": "GHSA-2",
            "summary": "Medium issue in A",
            "details": "Details about GHSA-2",
            "database_specific": { "severity": "MODERATE" }
        }"#;
        Mock::given(method("GET"))
            .and(path("/v1/vulns/GHSA-2"))
            .respond_with(ResponseTemplate::new(200).set_body_string(ghsa2))
            .mount(&mock_server)
            .await;

        let ghsa3 = r#"{
            "id": "GHSA-3",
            "summary": "Low issue in B",
            "details": "Details about GHSA-3",
            "database_specific": { "severity": "LOW" }
        }"#;
        Mock::given(method("GET"))
            .and(path("/v1/vulns/GHSA-3"))
            .respond_with(ResponseTemplate::new(200).set_body_string(ghsa3))
            .mount(&mock_server)
            .await;
    });

    let osv_url = mock_server.uri() + "/v1/querybatch";

    // 2. Setup Temp File with complex fixture
    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
    let lock_path = temp_dir.path().join("package-lock.json");

    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/complex.package-lock.json");

    std::fs::copy(fixture_path, &lock_path).expect("failed to copy fixture");

    // 3. Scan
    let result = scan_lockfile(
        &lock_path,
        ScanOptions {
            offline: false,
            show_details: true,
            osv_api_url: Some(osv_url),
            metrics: None,
        },
    )
    .expect("Scan failed");

    // 4. Generate Output
    let output = result.display(OutputFormat::Human);

    // 5. Normalization
    // Replace temp dir path with [TEMP]
    let normalized = output.replace(temp_dir.path().to_string_lossy().as_ref(), "[TEMP]");

    // 6. Assert Snapshot
    assert_snapshot!(normalized);
}

#[test]
fn test_report_output_duplicates() {
    // 1. Setup Mock Server
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mock_server = rt.block_on(wiremock::MockServer::start());

    // Mock response for a-lib
    let body = r#"{
        "results": [
            {
                "vulns": [
                    {
                        "id": "GHSA-1",
                        "summary": "Critical issue in A",
                        "details": "Details about GHSA-1",
                        "database_specific": { "severity": "CRITICAL" }
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

        // Details
        let ghsa1 = r#"{
            "id": "GHSA-1",
            "summary": "Critical issue in A",
            "details": "Details about GHSA-1",
            "database_specific": { "severity": "CRITICAL" }
        }"#;
        Mock::given(method("GET"))
            .and(path("/v1/vulns/GHSA-1"))
            .respond_with(ResponseTemplate::new(200).set_body_string(ghsa1))
            .mount(&mock_server)
            .await;
    });

    let osv_url = mock_server.uri() + "/v1/querybatch";

    // 2. Setup Temp File
    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
    let lock_path = temp_dir.path().join("package-lock.json");
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/duplicate.package-lock.json");
    std::fs::copy(fixture_path, &lock_path).expect("failed to copy fixture");

    // 3. Scan
    // We disable details fetching to test pure deduplication logic on input side,
    // but enabling it tests report grouping logic too.
    let result = scan_lockfile(
        &lock_path,
        ScanOptions {
            offline: false,
            show_details: true,
            osv_api_url: Some(osv_url),
            metrics: None,
        },
    )
    .expect("Scan failed");

    // 4. Generate Output
    let output = result.display(OutputFormat::Human);
    let normalized = output.replace(temp_dir.path().to_string_lossy().as_ref(), "[TEMP]");

    // 5. Assert: a-lib should appear ONCE.
    assert_snapshot!(normalized);
    // Also explicitly check vulnerabilities count
    assert_eq!(
        result.vulnerabilities.len(),
        1,
        "Should be deduplicated to 1 vulnerability group"
    );
}

#[test]
fn test_report_grouping_logic() {
    // This test manually constructs ScanResult with duplicates to verify REPORT layer merging,
    // ensuring we are not solely relying on scanner deduplication.

    use semver::VersionReq;
    use veil_guardian::models::{Advisory, Ecosystem};
    use veil_guardian::report::{ScanResult, Vulnerability};

    let adv1 = Advisory {
        id: "GHSA-dup-1".to_string(),
        crate_name: "manual-lib".to_string(),
        vulnerable_versions: VersionReq::parse("*").unwrap(),
        description: "Dup 1".to_string(),
        details: None,
        cache_status: Some("Hit (Fresh)".to_string()),
        last_fetched_at: None,
    };

    let vuln1 = Vulnerability {
        ecosystem: Ecosystem::Npm,
        package_name: "manual-lib".to_string(),
        version: "1.0.0".to_string(),
        advisories: vec![adv1.clone()],
        locations: vec!["path/to/lock_A".to_string()],
    };

    let vuln2 = Vulnerability {
        ecosystem: Ecosystem::Npm,
        package_name: "manual-lib".to_string(),
        version: "1.0.0".to_string(),
        advisories: vec![adv1.clone()], // Duplicate advisory, should merge
        locations: vec!["path/to/lock_B".to_string()], // Different location, should append
    };

    let scan_result = ScanResult {
        vulnerabilities: vec![vuln1, vuln2],
        scanned_crates: 10,
    };

    let output = scan_result.display(OutputFormat::Human);

    // Check that we only have one block for manual-lib
    assert_eq!(
        output.matches("manual-lib v1.0.0 (npm)").count(),
        1,
        "Should appear exactly once"
    );

    // Check that locations were merged
    assert!(
        output.contains("path/to/lock_A"),
        "Should contain first path"
    );
    assert!(
        output.contains("path/to/lock_B"),
        "Should contain second path"
    );

    // Check that advisory appears once (dedup by ID)
    assert_eq!(
        output.matches("GHSA-dup-1").count(),
        1,
        "Advisory should appear exactly once"
    );

    // Optional snapshot
    assert_snapshot!(output);
}
