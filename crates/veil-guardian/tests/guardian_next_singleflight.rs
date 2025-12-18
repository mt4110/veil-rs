use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use veil_guardian::guardian_next::net::SingleFlight;
use veil_guardian::guardian_next::CacheKey;

#[tokio::test]
async fn singleflight_runs_only_once() {
    let sf: SingleFlight<CacheKey, usize> = SingleFlight::new();
    let calls = Arc::new(AtomicUsize::new(0));

    let key = CacheKey("k".into());
    let mut tasks = vec![];

    for _ in 0..10 {
        let sf2 = sf.clone();
        let k2 = key.clone();
        let c2 = calls.clone();
        tasks.push(tokio::spawn(async move {
            sf2.do_call(k2, move || async move {
                tokio::time::sleep(std::time::Duration::from_millis(50)).await;
                c2.fetch_add(1, Ordering::SeqCst);
                42
            })
            .await
        }));
    }

    let results = futures::future::join_all(tasks).await;
    for r in results {
        assert_eq!(r.unwrap(), 42);
    }
    assert_eq!(calls.load(Ordering::SeqCst), 1);
}
