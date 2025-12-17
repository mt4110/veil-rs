use std::sync::atomic::Ordering;
use std::sync::{Arc, Barrier};
use std::time::Duration;
use tempfile::tempdir;
use veil_guardian::providers::osv::{
    details_store::DetailsStore,
    net::{NetConfig, Sleeper, TokioSleeper},
    OsvClient,
};
use veil_guardian::Metrics;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

#[test]
fn respects_max_in_flight_concurrency_gate() {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap();

    let (mock, url) = rt.block_on(async {
        let mock = MockServer::start().await;
        let url = mock.uri();

        // Prepare delayed responses to create overlap.
        let n = 20usize;
        for i in 0..n {
            let id = format!("OSV-TEST-{}", i);
            Mock::given(method("GET"))
                .and(path(format!("/vulns/{}", id)))
                .respond_with(
                    ResponseTemplate::new(200)
                        .set_delay(Duration::from_millis(200))
                        .set_body_json(serde_json::json!({ "id": id })),
                )
                .mount(&mock)
                .await;
        }
        (mock, url)
    });

    let dir = tempdir().unwrap();
    let store = DetailsStore::new(Some(dir.path().to_path_buf()));

    let metrics = Arc::new(Metrics::new());

    let mut net = NetConfig::default();
    net.concurrency.max_in_flight = 2;
    // OsvClient uses its own runtime, so we are fine.

    let sleeper: Arc<dyn Sleeper> = Arc::new(TokioSleeper);

    let client = Arc::new(OsvClient::new_custom_with_net_and_metrics(
        false,
        store,
        Some(url),
        net,
        sleeper,
        Some(metrics.clone()),
    ));

    let n = 20usize;
    let mut handles = Vec::with_capacity(n);

    // Use barriers to sync start if possible, but standard threads + OsvClient (blocking) is simple.
    let barrier = Arc::new(Barrier::new(n));

    for i in 0..n {
        let id = format!("OSV-TEST-{}", i);
        let c = client.clone();
        let b = barrier.clone();
        handles.push(std::thread::spawn(move || {
            b.wait();
            let _ = c.fetch_vuln_details(&id);
        }));
    }

    for h in handles {
        let _ = h.join();
    }

    let max = metrics.max_concurrency.load(Ordering::Relaxed);
    assert!(
        max <= 2,
        "max_concurrency exceeded max_in_flight: max_concurrency={}, max_in_flight=2",
        max
    );

    // Ensure mock stays alive until here
    drop(mock);
}
