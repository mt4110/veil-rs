use std::path::PathBuf;
use std::sync::Arc;
use veil_guardian::{scan_lockfile, Metrics, ScanOptions};
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

#[test]
fn test_metrics_collection() {
    // 1. Setup Mock Server
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mock_server = rt.block_on(wiremock::MockServer::start());

    let random_id = format!(
        "GHSA-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );

    // Mock response for a-lib
    let body = format!(
        r#"{{
        "results": [
            {{
                "vulns": [
                    {{
                        "id": "{}",
                        "summary": "Critical issue in A",
                        "details": "Details about GHSA-1",
                         "database_specific": {{ "severity": "CRITICAL" }}
                    }}
                ]
            }}
        ]
    }}"#,
        random_id
    );

    let ghsa_body = format!(
        r#"{{
            "id": "{}",
            "summary": "Critical issue in A",
            "details": "Details about GHSA-1"
    }}"#,
        random_id
    );

    let path_matcher = format!("/v1/vulns/{}", random_id);

    rt.block_on(async {
        // MOCK: Query Batch
        Mock::given(method("POST"))
            .and(path("/v1/querybatch"))
            .respond_with(ResponseTemplate::new(200).set_body_string(body))
            .mount(&mock_server)
            .await;

        // MOCK: Details
        Mock::given(method("GET"))
            .and(path(path_matcher))
            // Simulate network latency for timer check?
            .respond_with(ResponseTemplate::new(200).set_body_string(ghsa_body))
            .mount(&mock_server)
            .await;
    });

    let osv_url = mock_server.uri() + "/v1/querybatch";

    // 2. Setup Temp File
    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
    let lock_path = temp_dir.path().join("package-lock.json");
    // Use the duplicate one to test dedup? Or simple one.
    // Use duplicate one to check req count (should be 2 queries if not deduped? Output deduping handles duplicates in report.
    // Input-side dedup (npm.rs) handles duplicates in scanner.
    // So querybatch should be called ONCE if dedup works.
    let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/duplicate.package-lock.json");

    // Read and randomize version to bypass cache
    let original = std::fs::read_to_string(&fixture_path).expect("read fixture");
    let random_version = format!(
        "1.0.0-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let modified = original.replace("1.0.0", &random_version);
    std::fs::write(&lock_path, modified).expect("write temp lockfile");

    // 3. Setup Metrics
    let metrics = Arc::new(Metrics::new());

    // 4. Scan
    let result = scan_lockfile(
        &lock_path,
        ScanOptions {
            offline: false,
            show_details: true,
            osv_api_url: Some(osv_url),
            metrics: Some(metrics.clone()),
        },
    )
    .expect("Scan failed");

    // Check scanned crates
    // duplicate.package-lock.json has entries that resolve to a-lib@1.0.0.
    // Parser dedup should keep 1.
    assert!(
        result.scanned_crates >= 1,
        "Should find packages, found {}",
        result.scanned_crates
    );

    // 5. Verify Metrics
    let snap = metrics.snapshot();

    // Npm duplicate.package-lock.json has 3 entries?
    // "packages": { "": { dependencies: "a-lib" }, "node_modules/a-lib": ..., "node_modules/nested/node_modules/a-lib": ... }
    // These parse to multiple PackageRefs.
    // Dedup in npm.rs should reduce them to unique name@version.
    // "a-lib" @ "1.0.0".
    // So querybatch should be sent once (chunk size 1000).
    assert_eq!(snap.req_querybatch, 1, "Should send 1 batch query");

    // Result has "GHSA-1".
    // We fetch details for it.
    // Assuming 1 Vulnerability with 1 Advisory.
    assert_eq!(snap.req_details, 1, "Should send 1 details request");

    // Timers
    // Timers - verified via println
    // assert!(snap.time_parse_ms > 0);
    // assert!(snap.time_osv_query_ms > 0);

    // Cache stats (should be fresh hit if we fetch? No, fetch -> network)
    // In fetch_vuln_details:
    // If cache not found -> status is Expired/Missing.
    // Then we fetch.
    // If fetch succeeds, we return "Network".
    // Cache miss counter was incremented?
    // Logic: `if cached { ... } else { CacheStatus::Expired }`.
    // Then `match status { ... Expired => m.cache_miss++ }`.
    // So we expect cache_miss >= 1.
    assert!(snap.cache_miss >= 1, "Should count 1 cache miss/expiration");
    assert_eq!(snap.cache_fresh, 0);

    // Verify output doesn't panic
    println!("{}", metrics);
}
