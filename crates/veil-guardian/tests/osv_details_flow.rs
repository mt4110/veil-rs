use serde_json::json;
use std::time::{Duration, SystemTime};
use tempfile::tempdir;
use veil_guardian::providers::osv::{details::CachedVuln, details_store::DetailsStore, OsvClient};
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

#[tokio::test]
async fn test_osv_details_flow_fresh_skips_fetch() {
    let mock_server = MockServer::start().await;
    // We set the URL but expect NO connection
    let url = format!("{}/v1/querybatch", mock_server.uri());

    let dir = tempdir().unwrap();
    let dir_path = dir.path().to_path_buf();
    let store = DetailsStore::with_dir(&dir_path).unwrap();

    // 1. Pre-populate FRESH
    let id = "GHSA-fresh-123".to_string();
    let now = SystemTime::now();
    let cached_data = json!({"id": id, "summary": "Cached Fresh"});
    store
        .save(&CachedVuln::new(&id, now, cached_data.clone()))
        .unwrap();

    // 2. Run blocking client in separate thread to avoid reqwest panic
    // Double wrap: spawn_blocking (async) -> std::thread (no tokio context)
    let id_clone = id.clone();
    let url_clone = url.clone();
    let result = tokio::task::spawn_blocking(move || {
        std::thread::spawn(move || {
            let store = DetailsStore::with_dir(&dir_path).unwrap();
            let client = OsvClient::new_custom(false, Some(store), Some(url_clone));
            client.fetch_vuln_details(&id_clone)
        })
        .join()
        .unwrap()
    })
    .await
    .unwrap()
    .unwrap();

    let (val, status, _) = result;
    // OsvClient now returns String status, "Hit (Fresh)" or "Network"
    assert_eq!(status, "Hit (Fresh)");
    assert_eq!(val["summary"], "Cached Fresh");
}

#[tokio::test]
async fn test_osv_details_flow_expired_triggers_fetch() {
    let mock_server = MockServer::start().await;
    let url = format!("{}/v1/querybatch", mock_server.uri());

    let dir = tempdir().unwrap();
    let dir_path = dir.path().to_path_buf();
    let store = DetailsStore::with_dir(&dir_path).unwrap();

    // 1. Pre-populate EXPIRED
    let id = "GHSA-expired-999".to_string();
    let old = SystemTime::now() - Duration::from_secs(15 * 24 * 60 * 60);
    let old_data = json!({"id": id, "summary": "Old Data"});
    store.save(&CachedVuln::new(&id, old, old_data)).unwrap();

    // 2. Mock Expectation
    let new_data = json!({"id": id, "summary": "New Data"});
    Mock::given(method("GET"))
        .and(path(format!("/v1/vulns/{}", id)))
        .respond_with(ResponseTemplate::new(200).set_body_json(&new_data))
        .mount(&mock_server)
        .await;

    // 3. Blocking Run
    let id_clone = id.clone();
    let url_clone = url.clone();
    let result = tokio::task::spawn_blocking(move || {
        std::thread::spawn(move || {
            let store = DetailsStore::with_dir(&dir_path).unwrap();
            let client = OsvClient::new_custom(false, Some(store), Some(url_clone));
            client.fetch_vuln_details(&id_clone)
        })
        .join()
        .unwrap()
    })
    .await
    .unwrap()
    .unwrap();

    let (val, status, _) = result;
    // Note: status will be NETWORK after update since it fetched?
    // "Network".to_string()
    assert_eq!(status, "Network");
    assert_eq!(val["summary"], "New Data");

    // Verify disk updated
    let loaded = store.load(&id).unwrap();
    assert_eq!(loaded.vuln["summary"], "New Data");
}

#[tokio::test]
async fn test_osv_details_offline_uses_stale() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path().to_path_buf();
    let store = DetailsStore::with_dir(&dir_path).unwrap();

    // 1. Pre-populate STALE
    let id = "GHSA-stale-000".to_string();
    let stale_time = SystemTime::now() - Duration::from_secs(48 * 60 * 60);
    let data = json!({"id": id, "summary": "Stale Data"});
    store.save(&CachedVuln::new(&id, stale_time, data)).unwrap();

    // 2. Blocking Run
    let id_clone = id.clone();
    let result = tokio::task::spawn_blocking(move || {
        std::thread::spawn(move || {
            // Re-open store in thread
            let store = DetailsStore::with_dir(&dir_path).unwrap();
            let client = OsvClient::new_custom(true, Some(store), None); // Offline=true
            client.fetch_vuln_details(&id_clone)
        })
        .join()
        .unwrap()
    })
    .await
    .unwrap()
    .unwrap();

    let (val, status, _) = result;
    assert_eq!(status, "Hit (Stale)");
    assert_eq!(val["summary"], "Stale Data");
}
