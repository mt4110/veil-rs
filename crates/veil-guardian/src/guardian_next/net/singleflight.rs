use std::collections::HashMap;
use std::hash::Hash;
use std::sync::{Arc, Mutex};
use tokio::sync::Notify;

struct InFlight<V> {
    notify: Arc<Notify>,
    result: Arc<tokio::sync::Mutex<Option<V>>>,
}

#[derive(Clone)]
pub struct SingleFlight<K, V> {
    inner: Arc<Mutex<HashMap<K, InFlight<V>>>>,
}

impl<K, V> SingleFlight<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 同一キーは先頭だけが producer を実行。後続は notify 待ち。
    pub async fn do_call<F, Fut>(&self, key: K, producer: F) -> V
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = V>,
    {
        // fast path: 既にin-flightがある
        let (notify, result_mutex, is_leader) = {
            let mut map = self.inner.lock().unwrap();
            if let Some(inf) = map.get(&key) {
                (inf.notify.clone(), inf.result.clone(), false)
            } else {
                let inf = InFlight {
                    notify: Arc::new(Notify::new()),
                    result: Arc::new(tokio::sync::Mutex::new(None)),
                };
                let notify = inf.notify.clone();
                let result = inf.result.clone();
                map.insert(key.clone(), inf);
                (notify, result, true)
            }
        };

        if is_leader {
            let v = producer().await;
            {
                let mut slot = result_mutex.lock().await;
                *slot = Some(v.clone());
            }
            // map から削除して通知
            {
                let mut map = self.inner.lock().unwrap();
                map.remove(&key);
            }
            notify.notify_waiters();
            v
        } else {
            notify.notified().await;
            let slot = result_mutex.lock().await;
            slot.clone().expect("singleflight: notified but no result")
        }
    }
}

impl<K, V> Default for SingleFlight<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone,
{
    fn default() -> Self {
        Self::new()
    }
}
