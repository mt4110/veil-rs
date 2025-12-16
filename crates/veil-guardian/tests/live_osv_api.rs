use std::env;
use veil_guardian::models::{Ecosystem, PackageRef};
use veil_guardian::providers::osv::OsvClient;

// This test hits the REAL OSV API.
// It is disabled by default to keep CI fast and deterministic (Network-Zero).
// Run with: VEIL_LIVE_TESTS=1 cargo test --test live_osv_api
#[test]
fn test_live_osv_lodash_check() {
    if env::var("VEIL_LIVE_TESTS").is_err() {
        println!("Skipping live test. Set VEIL_LIVE_TESTS=1 to run.");
        return;
    }

    // OsvClient::new creates its own internal Tokio runtime.
    // We must NOT run this inside #[tokio::test] or we'll get "Cannot start a runtime from within a runtime"
    // when OsvClient calls block_on.
    // Default params: offline: false, api_url: None (default), metrics: None, cache_dir: None
    let client = OsvClient::new(false, None, None, None);

    let pkg = PackageRef {
        ecosystem: Ecosystem::Npm,
        name: "lodash".to_string(),
        version: "4.17.15".to_string(), // Known vulnerable version
    };

    println!("Fetching live data for lodash@4.17.15...");
    let results = client
        .check_packages(&[pkg], false)
        .expect("Failed to fetch from OSV API");

    assert!(
        !results.is_empty(),
        "Expected vulnerabilities for lodash 4.17.15"
    );

    let vuln = &results[0];
    assert_eq!(vuln.package_name, "lodash");
    assert!(!vuln.advisories.is_empty());

    // Verify we got at least one GHSA ID
    let has_ghsa = vuln.advisories.iter().any(|a| a.id.starts_with("GHSA-"));
    assert!(
        has_ghsa,
        "Expected at least one GHSA advisory, found: {:?}",
        vuln.advisories
    );

    println!(
        "Live Test Success: Found {} advisories for lodash.",
        vuln.advisories.len()
    );
}
