use ignore::WalkBuilder;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use veil_config::Config;
use veil_core::{scan_content, try_get_all_rules, Finding};

#[test]
fn jp_address_rule_uses_prefecture_city_block_validator() {
    let config = Config::default();
    let rules = try_get_all_rules(&config, vec![]).unwrap();
    let rule = rules
        .iter()
        .find(|rule| rule.id == "pii.jp.address.prefecture_heuristic")
        .expect("default JP address rule should be loaded");

    assert_eq!(
        rule.validator_id.as_deref(),
        Some("jp_address_prefecture_city_block")
    );
    assert!(
        rule.validator.is_some(),
        "JP address validator should resolve for the built-in rule"
    );
}

#[test]
fn jp_pii_positive_fixtures_match_expected_rules() {
    let fixture_dir = workspace_root()
        .join("tests")
        .join("fixtures")
        .join("jp_pii")
        .join("positive");

    let config = Config::default();
    let rules = try_get_all_rules(&config, vec![]).unwrap();

    for path in fixture_paths(&fixture_dir) {
        let content = fs::read_to_string(&path).unwrap();
        let expected = expected_rules(&content);
        assert!(
            !expected.is_empty(),
            "positive fixture must declare at least one '# EXPECT:' rule: {}",
            path.display()
        );

        let findings = scan_content(&content, &path, &rules, &config);
        let observed: BTreeSet<_> = findings
            .iter()
            .map(|finding| finding.rule_id.as_str())
            .collect();

        for rule_id in expected {
            assert!(
                observed.contains(rule_id.as_str()),
                "expected rule '{}' in {}, observed: {:?}\nfindings:\n{}",
                rule_id,
                path.display(),
                observed,
                format_findings(&findings)
            );
        }
    }
}

#[test]
fn jp_pii_negative_fixtures_have_no_findings() {
    let fixture_dir = workspace_root()
        .join("tests")
        .join("fixtures")
        .join("jp_pii")
        .join("negative");

    let config = Config::default();
    let rules = try_get_all_rules(&config, vec![]).unwrap();

    for path in fixture_paths(&fixture_dir) {
        let content = fs::read_to_string(&path).unwrap();
        let findings = scan_content(&content, &path, &rules, &config);

        assert!(
            findings.is_empty(),
            "negative fixture should produce zero findings: {}\nfindings:\n{}",
            path.display(),
            format_findings(&findings)
        );
    }
}

fn fixture_paths(dir: &Path) -> Vec<PathBuf> {
    let mut paths: Vec<_> = WalkBuilder::new(dir)
        .build()
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry
                .file_type()
                .is_some_and(|file_type| file_type.is_file())
        })
        .map(|entry| entry.into_path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("txt"))
        .collect();
    paths.sort();
    paths
}

fn expected_rules(content: &str) -> Vec<String> {
    content
        .lines()
        .filter_map(|line| line.strip_prefix("# EXPECT:"))
        .map(str::trim)
        .filter(|rule_id| !rule_id.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn format_findings(findings: &[Finding]) -> String {
    findings
        .iter()
        .map(|finding| {
            format!(
                "{}:{}:{}:{}",
                finding.rule_id, finding.line_number, finding.score, finding.matched_content
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}
