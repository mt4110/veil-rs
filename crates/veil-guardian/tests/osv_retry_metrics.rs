use std::sync::Arc;
use std::time::Duration;
use tempfile::tempdir;
use veil_guardian::providers::osv::{
    details_store::DetailsStore,
    net::{NetConfig, RecordingSleeper, RetryPolicy},
    OsvClient,
};
use veil_guardian::Metrics;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

#[tokio::test]
async fn verify_metrics_on_retry_and_success() {
    let mock = MockServer::start().await;
    let url = mock.uri();
    let dir = tempdir().unwrap();
    let id = "GHSA-metrics-retry".to_string();

    // 1. Setup: 500 (Fatal/Retryable?) -> 500 logic is RetryClass::Fatal by default?
    // Wait, in retry.rs, 500 is typically retryable unless configured otherwise.
    // Let's check classify_error/response defaults. usually 500/502/503/504 are retryable.
    // Let's use 429 which is definitely retryable.

    // First request: 429, Retry-After: 1
    Mock::given(method("GET"))
        .and(path(format!("/vulns/{}", id)))
        .respond_with(ResponseTemplate::new(429).insert_header("Retry-After", "1"))
        .up_to_n_times(1)
        .mount(&mock)
        .await;

    // Second request: 200 OK
    Mock::given(method("GET"))
        .and(path(format!("/vulns/{}", id)))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": id,
            "modified": "2021-01-01T00:00:00Z"
        })))
        .mount(&mock)
        .await;

    let sleeper = Arc::new(RecordingSleeper::new());
    let metrics = Arc::new(Metrics::new());

    let net = NetConfig {
        retry: RetryPolicy {
            max_attempts: 3,
            jitter_factor: 0.0,
            ..Default::default()
        },
        total_budget: Duration::from_secs(10),
        ..Default::default()
    };

    let id2 = id.clone();
    let url2 = url.clone();
    let sleeper2 = sleeper.clone();
    let metrics2 = metrics.clone();

    tokio::task::spawn_blocking(move || {
        let store = DetailsStore::new(Some(dir.path().to_path_buf()));
        // Use new_custom_with_net_and_metrics
        let client = OsvClient::new_custom_with_net_and_metrics(
            false,
            store,
            Some(url2),
            net,
            sleeper2,
            Some(metrics2),
        );
        client.fetch_vuln_details(&id2)
    })
    .await
    .unwrap()
    .expect("Fetch should succeed after retry");

    // Assertions
    let snap = metrics.snapshot();

    // 1. net_limit_exceeded (429 happened once)
    assert_eq!(snap.net_limit_exceeded, 1, "Should count 1 429 error");

    // 2. net_retry_attempts (1 retry occurred)
    assert_eq!(snap.net_retry_attempts, 1, "Should count 1 retry attempt");

    // 3. net_retry_sleep_ms (slept for 1s = 1000ms)
    assert_eq!(snap.net_retry_sleep_ms, 1000, "Should track sleep duration");

    // 4. net_fetched (eventually succeeded)
    assert_eq!(snap.net_fetched, 1, "Should count 1 successful fetch");

    // 5. req_details (2 total requests made: 1 fail + 1 success)
    assert_eq!(snap.req_details, 2, "Should count 2 total requests");
}
