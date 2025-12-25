use std::io::Cursor;
use veil_core::metrics::aggregate::aggregate_events;
use veil_core::metrics::reason::{ReasonCode, ReasonEventV1};

#[test]
fn test_golden_metrics_deterministic_output() {
    // 1. Prepare deterministic input events (unordered in time/stream)
    use veil_core::metrics::hint::HintCode;

    let events = vec![
        ReasonEventV1 {
            v: 1,
            ts: "2025-12-25T10:00:00Z".to_string(),
            reason_code: ReasonCode::ConfigMissingRequired,
            op: "load_config".to_string(),
            outcome: "fail".to_string(),
            taxon: Some("config_error".to_string()),
            detail: "missing field X".to_string(),
            hint_codes: vec![HintCode::UpgradeTool], // Use a valid hint code
        },
        ReasonEventV1 {
            v: 1,
            ts: "2025-12-25T10:01:00Z".to_string(),
            reason_code: ReasonCode::Offline,
            op: "check_update".to_string(),
            outcome: "fail".to_string(),
            taxon: Some("network_error".to_string()),
            detail: "no route to host".to_string(),
            hint_codes: vec![HintCode::CheckNetwork],
        },
        ReasonEventV1 {
            v: 1,
            ts: "2025-12-25T10:02:00Z".to_string(),
            reason_code: ReasonCode::ConfigMissingRequired, // Duplicate reason to check aggregation
            op: "load_config_retry".to_string(),
            outcome: "fail".to_string(),
            taxon: Some("config_error".to_string()),
            detail: "still missing field X".to_string(),
            hint_codes: vec![],
        },
        // Dogfood event (should be filtered for hints)
        ReasonEventV1 {
            v: 1,
            ts: "2025-12-25T10:03:00Z".to_string(),
            reason_code: ReasonCode::Unexpected,
            op: "audit.scorecard".to_string(),
            outcome: "fail".to_string(),
            taxon: None,
            detail: "infra fail".to_string(),
            hint_codes: vec![HintCode::RetryLater],
        },
    ];

    // 2. Serialize to JSONL (simulating file read)
    let mut jsonl_data = Vec::new();
    for evt in events {
        serde_json::to_writer(&mut jsonl_data, &evt).unwrap();
        jsonl_data.push(b'\n');
    }
    // Add some noise (empty lines, invalid json) to test robustness
    jsonl_data.extend_from_slice(b"\n{BrokenJson\n");

    // 3. Run aggregation
    let cursor = Cursor::new(jsonl_data);
    let (metrics, stats) = aggregate_events(cursor);

    // 4. Verify Stats
    assert_eq!(stats.valid_events, 4);
    assert_eq!(stats.parse_errors, 1); // The broken JSON line

    // 5. Verify Metrics Content
    let counts = &metrics.metrics.counts_by_reason;
    assert_eq!(counts.get("config_missing_required"), Some(&2));
    assert_eq!(counts.get("offline"), Some(&1));
    assert_eq!(counts.get("unexpected"), Some(&1)); // Dogfood failure is counted in reasons

    let hint_counts = &metrics.metrics.counts_by_hint;
    assert_eq!(hint_counts.get("upgrade_tool"), Some(&1));
    assert_eq!(hint_counts.get("check_network"), Some(&1));
    // Verify dogfood filter: REMOVED for Phase 12 stability. "retry_later" SHOULD be here.
    assert_eq!(hint_counts.get("retry_later"), Some(&1));

    // 6. Verify Deterministic JSON Output (Keys must be sorted)
    let output_json = serde_json::to_string(&metrics).unwrap();
    // BTreeMap serializes keys in order.
    // "config_missing_required" comes before "offline".
    let expected_order = output_json.find("config_missing_required").unwrap() < output_json.find("offline").unwrap();
    assert!(expected_order, "metrics output should be sorted by keys");

    println!("Golden Metrics Output: {}", output_json);
}
