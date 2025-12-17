use std::sync::Arc;
use veil_guardian::guardian_next::net::ReqwestHttpClient;
use veil_guardian::guardian_next::{GuardianNext, GuardianNextConfig};

use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn retry_on_500_then_success() {
    let server = MockServer::start().await;

    // 1回目 500
    // 2回目 200 (Default Fallback - Low Priority via LIFO)
    Mock::given(method("GET"))
        .and(path("/v1/vulns/GHSA-TEST"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(serde_json::json!({"id":"GHSA-TEST"})),
        )
        .mount(&server)
        .await;

    // 1回目 500 (High Priority via LIFO)
    Mock::given(method("GET"))
        .and(path("/v1/vulns/GHSA-TEST"))
        .respond_with(ResponseTemplate::new(500))
        .up_to_n_times(1)
        .mount(&server)
        .await;

    // Create temporary cache dir
    let tmp_dir = tempfile::tempdir().unwrap();

    let cfg = GuardianNextConfig {
        offline: false,
        http_timeout_ms: 2000,
        cache_dir: tmp_dir.path().to_path_buf(),
        ..Default::default()
    };

    let http = Arc::new(ReqwestHttpClient::new().unwrap());
    let g = GuardianNext::new(cfg, http).unwrap();

    // Since GuardianNext uses hardcoded URL in `get_osv_details_json` (api.osv.dev),
    // we cannot easily test it with wiremock unless we modify GuardianNext to accept base URL or use `get_json_with_policy`.
    // But `get_json_with_policy` is private.
    // However, `get_osv_details_json` is the public API.
    // For this test to work without modifying `GuardianNext`, we would need to mock `HttpClient`.
    // But we are using `ReqwestHttpClient`.
    //
    // Option A: Make `get_json_with_policy` public for crate or tests.
    // Option B: Implementing a MockHttpClient that delegates to wiremock? No, wiremock is HTTP server.
    // Option C: Changing `GuardianNext` to accept a base_url in config or constructor used for OSV.

    // The user's provided test code used:
    // let url = format!("{}/v1/vulns/GHSA-TEST", &server.uri());
    // let key = ...
    // let (payload, _) = g.get_json_with_policy(&key, &url).await.unwrap();

    // But `get_json_with_policy` is private in the provided code snippet.
    // I should make `get_json_with_policy` `pub(crate)` in `fetcher.rs`.
    // I will Assume I made it pub(crate) (I need to update `fetcher.rs`).

    let url = format!("{}/v1/vulns/GHSA-TEST", &server.uri());
    let key = veil_guardian::guardian_next::CacheKey("osv:vuln:GHSA-TEST".into());

    let (payload, _outcome) = g.get_json_with_policy(&key, &url).await.unwrap();

    assert_eq!(payload["id"], "GHSA-TEST");
}
