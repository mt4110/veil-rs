use std::path::Path;
use veil_config::Config;
use veil_core::{scan_content, try_get_all_rules};

fn names(content: &str, path: &str) -> usize {
    let config = Config::default();
    let rules = try_get_all_rules(&config, vec![]).unwrap();
    scan_content(content, Path::new(path), &rules, &config)
        .iter()
        .filter(|finding| finding.rule_id == "pii.person.name.keyword")
        .count()
}

#[test]
fn name_context_rejects_ci_actions_and_compound_prose() {
    for (content, path) in [
        ("      - name: Check formatting", "workflow.yml"),
        ("      - name: Run clippy", "workflow.yml"),
        (
            "- Docker: published-port/container-name mapping and restart action.",
            "guide.md",
        ),
    ] {
        assert_eq!(names(content, path), 0, "synthetic non-person case");
    }
}

#[test]
fn name_context_preserves_person_labels_across_formats() {
    for path in ["workflow.yml", "guide.md", "profile.txt", "source.rs"] {
        for content in [
            "- name: Alice Morgan",
            "Name: Alice Morgan",
            "name = 'Hanako Sato'",
            "Name Alice Morgan",
            "氏名: 山田 太郎",
            "お名前：佐藤花子",
            "full-name: Alice Morgan",
            "container-name: Alice Morgan",
            "container-name Alice Morgan",
            "- name: Check Morgan",
            "- name: Run Morgan",
            "mycontainer-name Mapping Morgan",
            "container-name Mappington Morgan",
            "container-name mapping; Name: Alice Morgan",
            "- name: Check Formatting; お名前：佐藤花子",
            "- name: Check formatting; Name: Alice Morgan",
            "- name: Run clippy; 氏名: 山田 太郎",
        ] {
            assert!(names(content, path) > 0, "synthetic personal-name case");
        }
    }
}

#[test]
fn name_context_preserves_other_rules_on_the_same_line() {
    let config = Config::default();
    let rules = try_get_all_rules(&config, vec![]).unwrap();
    let content = "- name: Run clippy; token=AKIAIOSFODNN7EXAMPLE; bind=127.0.0.1";
    let findings = scan_content(content, Path::new("workflow.yml"), &rules, &config);
    assert!(findings
        .iter()
        .any(|finding| finding.rule_id.contains("aws")));
    assert!(findings
        .iter()
        .any(|finding| finding.rule_id == "pii.net.ipv4"));
    assert!(!findings
        .iter()
        .any(|finding| finding.rule_id == "pii.person.name.keyword"));
}
