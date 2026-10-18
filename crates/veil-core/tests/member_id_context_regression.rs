use std::path::Path;
use veil_config::Config;
use veil_core::{scan_content, try_get_all_rules};

fn ids(content: &str) -> usize {
    let config = Config::default();
    let rules = try_get_all_rules(&config, vec![]).unwrap();
    scan_content(content, Path::new("synthetic.rs"), &rules, &config)
        .iter()
        .filter(|finding| finding.rule_id == "pii.id.customer_member.keyword")
        .count()
}

#[test]
fn member_id_context_rejects_uid_accessor_in_matches_expression() {
    let content = concat!(
        "|| matches!((task.and_then(|item| item.user_id()), active_uid), ",
        "(Some(owner), Some(active)) if owner == active),",
    );
    assert_eq!(ids(content), 0);
}

#[test]
fn member_id_context_preserves_labeled_ids_and_literals() {
    for content in [
        "customer_id: ABC12345",
        "member_id = MEM-1234",
        "user_id: USER-1234",
        "顧客番号: CUST-1234",
        "会員ID: MEMBER-1234",
        "会員番号: MEM-1234",
        "メンバーID: MEM-1234",
        "ユーザーID: USER-1234",
        "customer_id (legacy): ABC12345",
        "let sample = \"user_id: USER-1234\";",
        "// user_id: USER-1234",
        "matches!((task.user_id()), Some(owner)); user_id: USER-1234",
        "matches!((task.user_id()), Some(owner)); member_id: MEM-1234",
        "matches!((task.user_id()), Some(owner)); 顧客番号: CUST-1234",
        "matches!((task.user_id()), \"MEM-1234\")",
        "matches!((task.user_id()), MEMBER1234)",
    ] {
        assert!(
            ids(content) > 0,
            "synthetic personal ID must survive: {content}"
        );
    }
}

#[test]
fn member_id_context_preserves_test_ipv4() {
    let config = Config::default();
    let rules = try_get_all_rules(&config, vec![]).unwrap();
    let findings = scan_content(
        "user_id: USER-1234; bind=127.0.0.1",
        Path::new("test.rs"),
        &rules,
        &config,
    );
    assert!(findings
        .iter()
        .any(|finding| finding.rule_id == "pii.net.ipv4"));
    assert!(findings
        .iter()
        .any(|finding| finding.rule_id == "pii.id.customer_member.keyword"));
}

#[test]
fn member_id_context_keeps_the_actual_id_after_an_accessor() {
    let config = Config::default();
    let rules = try_get_all_rules(&config, vec![]).unwrap();
    for label in ["customer_id", "member_id", "user_id", "顧客番号", "会員ID"] {
        let content = format!(
            "matches!((task.and_then(|item| item.user_id()), active_uid), (Some(owner), Some(active)) if owner == active); {label}: RECORD-1234"
        );
        let findings = scan_content(&content, Path::new("synthetic.rs"), &rules, &config);
        assert!(
            findings.iter().any(|finding| {
                finding.rule_id == "pii.id.customer_member.keyword"
                    && finding.matched_content.contains("RECORD-1234")
            }),
            "the labeled value itself must remain detectable"
        );
    }
}
