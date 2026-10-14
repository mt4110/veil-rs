use std::collections::HashSet;
use std::path::Path;
use veil_config::Config;
use veil_core::{get_all_rules, rules::pack::load_rules_from_content, scan_content};

fn check_ipv4_rules(rules: &[veil_core::Rule]) {
    let config = Config::default();
    for candidate in ["0.0.0.0", "00.0.000.0", "0.0.0.0:8080"] {
        let findings = scan_content(candidate, Path::new("fixture.txt"), rules, &config);
        assert!(findings.is_empty(), "unspecified IPv4 must not be reported");
    }
    for candidate in [
        "0.0.0.1",
        "0.0.1.0",
        "0.1.0.0",
        "1.0.0.0",
        "10.23.45.67",
        "172.16.4.2",
        "192.168.3.4",
        "8.8.8.8",
        "127.0.0.1",
        "192.0.2.1",
        "255.255.255.255",
    ] {
        let findings = scan_content(candidate, Path::new("fixture.txt"), rules, &config);
        assert_eq!(
            findings.len(),
            1,
            "non-unspecified IPv4 detection must remain: {candidate}"
        );
    }
    let findings = scan_content(
        "bind=0.0.0.0 peer=10.23.45.67",
        Path::new("fixture.txt"),
        rules,
        &config,
    );
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].matched_content, "10.23.45.67");
}

#[test]
fn embedded_ipv4_excludes_only_unspecified() {
    let rules: Vec<_> = get_all_rules(&Config::default(), vec![])
        .into_iter()
        .filter(|r| r.id == "pii.net.ipv4")
        .collect();
    assert_eq!(rules.len(), 1);
    check_ipv4_rules(&rules);
}

#[test]
fn shipped_ipv4_variants_exclude_only_unspecified() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../veil");
    for relative in [
        "rules/default/pii.toml",
        "rules/default/pii_jp.toml",
        "rules_ja/default/pii_jp.toml",
        "rules/log/pii.toml",
    ] {
        let path = root.join(relative);
        let mut rules = Vec::new();
        load_rules_from_content(
            &std::fs::read_to_string(&path).unwrap(),
            &mut rules,
            &mut HashSet::new(),
            Some(&path),
        )
        .unwrap();
        rules.retain(|r| r.id == "pii.net.ipv4" || r.id == "log.pii.ipv4");
        assert_eq!(rules.len(), 1);
        check_ipv4_rules(&rules);
    }
}

#[test]
fn unspecified_filter_does_not_suppress_other_rules_on_the_same_line() {
    let mut rules: Vec<_> = get_all_rules(&Config::default(), vec![])
        .into_iter()
        .filter(|r| r.id == "pii.net.ipv4")
        .collect();
    let mut ids = rules.iter().map(|r| r.id.clone()).collect();
    load_rules_from_content(
        r#"[[rules]]
id = "fixture.sensitive"
description = "Synthetic sensitive marker"
pattern = 'SENSITIVE_FIXTURE'
severity = "HIGH"
score = 90
"#,
        &mut rules,
        &mut ids,
        None,
    )
    .unwrap();
    let findings = scan_content(
        "bind=0.0.0.0 token=SENSITIVE_FIXTURE",
        Path::new("fixture.txt"),
        &rules,
        &Config::default(),
    );
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].rule_id, "fixture.sensitive");
}
