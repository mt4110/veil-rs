use std::sync::{Arc, Barrier};
use std::thread;
use veil_guardian::providers::osv::OsvClient;
use veil_guardian::{
    models::{Ecosystem, PackageRef},
    Metrics,
};
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

#[test]
fn test_querybatch_coalesces_in_flight() {
    // 1. Setup Mock Server with DELAY
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mock_server = rt.block_on(wiremock::MockServer::start());

    let body = r#"{ "results": [ { "vulns": [] } ] }"#;

    rt.block_on(async {
        Mock::given(method("POST"))
            .and(path("/v1/querybatch"))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(body)
                    .set_delay(std::time::Duration::from_millis(3000)), // 3s delay
            )
            .mount(&mock_server)
            .await;
    });

    let api_url = mock_server.uri() + "/v1/querybatch";

    // 2. Setup Client
    let metrics = Arc::new(Metrics::new());
    // Create client (Arc specific to handle shared access in test? OsvClient is not Clone, but methods take &self)
    // We wrap OsvClient in Arc.
    let client = Arc::new(OsvClient::new(
        false,
        Some(api_url),
        Some(metrics.clone()),
        None,
    ));

    // 3. Spawn Threads
    let concurrency = 10;
    let barrier = Arc::new(Barrier::new(concurrency));
    let mut handles = vec![];

    // Shared random version for all threads (so they coalesce)
    let random_version = format!(
        "1.0.0-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );

    for _ in 0..concurrency {
        let c = client.clone();
        let b = barrier.clone();
        let ver = random_version.clone();
        handles.push(thread::spawn(move || {
            let pkg = PackageRef {
                ecosystem: Ecosystem::Npm,
                name: "heavy-lib".to_string(),
                version: ver,
            };

            // Wait for all threads to be ready
            b.wait();

            // Call check_packages (Sync, blocks on internal runtime)
            let _ = c.check_packages(&[pkg], false).expect("Scan failed");
        }));
    }

    // 4. Join
    for h in handles {
        h.join().unwrap();
    }

    // 5. Verify Metrics
    let snap = metrics.snapshot();
    println!("{}", metrics);

    assert!(
        snap.req_querybatch <= 2,
        "Should have few network requests (<=2), got {}",
        snap.req_querybatch
    );
    assert!(
        snap.coalesced_waiters >= (concurrency as u64 - 2),
        "Should have high coalesced events"
    );
}

#[test]
fn test_details_coalesces_in_flight() {
    // 1. Setup Mock Server with DELAY
    let rt = tokio::runtime::Runtime::new().unwrap();
    let mock_server = rt.block_on(wiremock::MockServer::start());

    let random_id = format!(
        "GHSA-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    let path_matcher = format!("/v1/vulns/{}", random_id);

    let body = format!(
        r#"{{
        "id": "{}",
        "summary": "Shared",
        "details": "Details"
    }}"#,
        random_id
    );

    rt.block_on(async {
        Mock::given(method("GET"))
            .and(path(path_matcher))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_string(body)
                    .set_delay(std::time::Duration::from_millis(3000)),
            )
            .mount(&mock_server)
            .await;
    });

    // Handle URL logic (base url ends with querybatch usually)
    let api_url = mock_server.uri() + "/v1/querybatch";

    let metrics = Arc::new(Metrics::new());
    let client = Arc::new(OsvClient::new(
        false,
        Some(api_url),
        Some(metrics.clone()),
        None,
    ));

    let concurrency = 10;
    let barrier = Arc::new(Barrier::new(concurrency));
    let mut handles = vec![];

    for _ in 0..concurrency {
        let c = client.clone();
        let b = barrier.clone();
        let id = random_id.clone();
        handles.push(thread::spawn(move || {
            b.wait();
            // Call fetch_vuln_details (Sync)
            let _ = c.fetch_vuln_details(&id).expect("Fetch failed");
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    let snap = metrics.snapshot();
    println!("{}", metrics);

    assert!(
        snap.req_details <= 2,
        "Should have few details requests (<=2), got {}",
        snap.req_details
    );
    assert!(
        snap.coalesced_waiters >= (concurrency as u64 - 2),
        "Should have high coalesced waiters"
    );
}
