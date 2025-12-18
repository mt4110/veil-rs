use std::sync::Arc;
use std::time::Duration;
use tempfile::tempdir;
use veil_guardian::providers::osv::{
    details_store::DetailsStore,
    net::{NetConfig, RecordingSleeper, RetryPolicy},
    OsvClient,
};
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

#[tokio::test]
async fn retry_on_500_records_backoff_sleeps() {
    let mock = MockServer::start().await;
    let url = mock.uri(); // Clean URL, OsvClient handles path

    // OsvClient expects base URL without /querybatch if possible, or handles logic.
    // The client code:
    // if base_url.ends_with("/querybatch") { replace } else { split/format }
    // If we pass mock.uri() which is e.g. http://127.0.0.1:xxx
    // It appends /vulns/{id}

    let dir = tempdir().unwrap();

    let id = "GHSA-retry-500".to_string();

    Mock::given(method("GET"))
        .and(path(format!("/vulns/{}", id)))
        .respond_with(ResponseTemplate::new(500))
        .expect(3) // 3 attempts (initial + 2 retries)
        .mount(&mock)
        .await;

    let sleeper = Arc::new(RecordingSleeper::new());

    let net = NetConfig {
        retry: RetryPolicy {
            max_attempts: 3,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(1),
            jitter_factor: 0.0,
        },
        total_budget: Duration::from_secs(10),
        ..Default::default()
    };

    let id2 = id.clone();
    let url2 = url.clone();
    let sleeper2 = sleeper.clone();

    // Run in blocking thread as OsvClient is sync-bridged internally for `fetch_vuln_details`
    let result = tokio::task::spawn_blocking(move || {
        // We need to use new_custom_with_net which is sync? No, it's just a constructor.
        // OsvClient::new_custom_with_net returns Self.
        let store = DetailsStore::new(Some(dir.path().to_path_buf()));
        let client = OsvClient::new_custom_with_net(false, store, Some(url2), net, sleeper2);
        client.fetch_vuln_details(&id2)
    })
    .await
    .unwrap();

    assert!(result.is_err());
    let sleeps = sleeper.sleeps();
    assert_eq!(sleeps.len(), 2);
    // 1st retry: 100ms * 2^0 = 100ms
    // 2nd retry: 100ms * 2^1 = 200ms
    assert_eq!(sleeps[0], Duration::from_millis(100));
    assert_eq!(sleeps[1], Duration::from_millis(200));
}

#[tokio::test]
async fn retry_after_is_respected() {
    let mock = MockServer::start().await;
    let url = mock.uri();
    let dir = tempdir().unwrap();

    let id = "GHSA-retry-429".to_string();

    // First request returns 429 with Retry-After: 2
    // Second request returns 200 OK (simulated, though here we verify failure behavior or success)

    // If we want to simulate success after retry:
    Mock::given(method("GET"))
        .and(path(format!("/vulns/{}", id)))
        .respond_with(ResponseTemplate::new(429).insert_header("Retry-After", "2"))
        .up_to_n_times(1)
        .mount(&mock)
        .await;

    Mock::given(method("GET"))
        .and(path(format!("/vulns/{}", id)))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": id,
            "modified": "2021-01-01T00:00:00Z"
        })))
        .mount(&mock)
        .await;

    // Wait, if we only mount 429, it will keep returning 429.
    // If we want to verify it slept 2s and retried.

    let sleeper = Arc::new(RecordingSleeper::new());
    let net = NetConfig {
        retry: RetryPolicy {
            max_attempts: 2, // Initial + 1 retry
            jitter_factor: 0.0,
            ..Default::default()
        },
        total_budget: Duration::from_secs(10),
        ..Default::default()
    };

    let id2 = id.clone();
    let url2 = url.clone();
    let sleeper2 = sleeper.clone();

    let result = tokio::task::spawn_blocking(move || {
        let store = DetailsStore::new(Some(dir.path().to_path_buf()));
        let client = OsvClient::new_custom_with_net(false, store, Some(url2), net, sleeper2);
        client.fetch_vuln_details(&id2)
    })
    .await
    .unwrap();

    assert!(result.is_ok()); // Should succeed after retry
    let sleeps = sleeper.sleeps();
    // Should have slept once for 2 seconds
    assert_eq!(sleeps, vec![Duration::from_secs(2)]);
}

#[tokio::test]
async fn retry_after_exceeds_budget_fails_fast() {
    let mock = MockServer::start().await;
    let url = mock.uri();
    let dir = tempdir().unwrap();

    let id = "GHSA-budget".to_string();
    Mock::given(method("GET"))
        .and(path(format!("/vulns/{}", id)))
        .respond_with(ResponseTemplate::new(429).insert_header("Retry-After", "99"))
        .mount(&mock)
        .await;

    let sleeper = Arc::new(RecordingSleeper::new());
    let net = NetConfig {
        total_budget: Duration::from_secs(1),
        retry: RetryPolicy {
            max_attempts: 3,
            ..Default::default()
        },
        ..Default::default()
    };

    let id2 = id.clone();
    let url2 = url.clone();
    let sleeper2 = sleeper.clone();

    let result = tokio::task::spawn_blocking(move || {
        let store = DetailsStore::new(Some(dir.path().to_path_buf()));
        let client = OsvClient::new_custom_with_net(false, store, Some(url2), net, sleeper2);
        client.fetch_vuln_details(&id2)
    })
    .await
    .unwrap();

    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("budget exceeded"));
    assert!(sleeper.sleeps().is_empty()); // Should NOT sleep if budget exceeded
}
