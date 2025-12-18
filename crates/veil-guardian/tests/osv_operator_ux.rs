use serde_json::json;
use std::fs;
use tempfile::tempdir;
use veil_guardian::providers::osv::{details_store::DetailsStore, OsvClient};
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

#[tokio::test]
async fn test_operator_message_includes_quarantine_note() {
    let mock_server = MockServer::start().await;
    let url = format!("{}/v1/querybatch", mock_server.uri());

    let dir = tempdir().unwrap();
    let dir_path = dir.path().to_path_buf();

    // 1. Setup CONFLICT: Create a FILE where v1 dir should be
    // <root>/vulns/v1
    let v1_path = dir_path.join("vulns").join("v1");
    fs::create_dir_all(v1_path.parent().unwrap()).unwrap();
    fs::write(&v1_path, "I am a file, not a directory").unwrap();

    // 2. Mock Fetch Success
    let id = "GHSA-conflict-123".to_string();
    let new_data = json!({"id": id, "summary": "Fetched Data"});
    Mock::given(method("GET"))
        .and(path(format!("/v1/vulns/{}", id)))
        .respond_with(ResponseTemplate::new(200).set_body_json(&new_data))
        .mount(&mock_server)
        .await;

    // 3. Run Client
    let id_clone = id.clone();
    let url_clone = url.clone();
    let result = tokio::task::spawn_blocking(move || {
        std::thread::spawn(move || {
            // DetailsStore::new will try to ensure dir, triggering quarantine logic immediately?
            // Wait, DetailsStore::new calls `fs::create_dir_all`.
            // If v1 is a file, `create_dir_all` might fail or handle it?
            // Actually `DetailsStore::new` calls `fs::create_dir_all` on `dir`.
            // And then `fs::create_dir_all(dir.join("vulns").join("v1"))`.
            // If that fails, `new` might panic or error if we were using it directly.
            // But `OsvClient::new_custom` takes `Option<DetailsStore>`.
            // Let's create store explicitly.

            // Wait, DetailsStore::new might fail to create the struct if `create_dir_all` fails.
            // But `ensure_v1_dir` logic I added is inside `load`.
            // The `new` method blindly calls `create_dir_all` on v1 path.
            // If v1 path is a file, `create_dir_all` errors on Unix.
            // So `DetailsStore::new` probably returns None or panics depending on implementation.
            // Let's check `DetailsStore::new` implementation in `details_store.rs`.
            // It uses `_ = fs::create_dir_all(...)`. It ignores the result!
            // So `new` succeeds. The file remains.

            let store = DetailsStore::with_dir(&dir_path).unwrap();

            // Now `OsvClient` fetch calls `store.load()`.
            // `load` calls `ensure_v1_dir`.
            // My change in Step 1 makes `ensure_v1_dir` detect file, quarantine it, create dir, and set conflict flag.

            let client = OsvClient::new_custom(false, Some(store), Some(url_clone));
            client.fetch_vuln_details(&id_clone)
        })
        .join()
        .unwrap()
        .unwrap() // Result<Result>
    })
    .await
    .unwrap();

    let (_, status_label, _) = result;

    // 4. Verify Operator Note
    assert!(
        status_label.contains("(quarantined: conflict)"),
        "Status '{}' should contain quarantine note",
        status_label
    );
    assert!(
        status_label.contains("Fetched"),
        "Status '{}' should indicate fetch happened",
        status_label
    );
}

#[tokio::test]
async fn test_operator_message_offline_remediation() {
    let dir = tempdir().unwrap();
    let dir_path = dir.path().to_path_buf();

    // 1. Setup CONFLICT
    let v1_path = dir_path.join("vulns").join("v1");
    fs::create_dir_all(v1_path.parent().unwrap()).unwrap();
    fs::write(&v1_path, "Conflict File").unwrap();

    // 2. Run Client OFFLINE
    let id = "GHSA-offline-conflict".to_string();
    let result = tokio::task::spawn_blocking(move || {
        std::thread::spawn(move || {
            let store = DetailsStore::with_dir(&dir_path).unwrap();
            let client = OsvClient::new_custom(true, Some(store), None); // offline=true
            client.fetch_vuln_details(&id)
        })
        .join()
        .unwrap()
    })
    .await
    .unwrap();

    // 3. Verify Error Message
    match result {
        Ok(_) => panic!("Should have failed offline with conflict"),
        Err(e) => {
            let msg = e.to_string();
            assert!(msg.contains("Offline: No details cached"), "Msg: {}", msg);
            assert!(msg.contains("(quarantined: conflict)"), "Msg: {}", msg);
            assert!(
                msg.contains("Hint: Run online to self-heal"),
                "Msg: {}",
                msg
            );
        }
    }
}
