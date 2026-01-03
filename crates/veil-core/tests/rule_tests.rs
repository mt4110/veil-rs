use ignore::WalkBuilder;
use std::fs;
use std::path::Path;
use veil_config::Config;
use veil_core::get_all_rules;
use veil_core::scan_content;

mod helpers;
use helpers::{fake_slack_bot_token, fake_slack_user_token};

#[test]
fn test_rules_from_fixtures() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap();
    let root_dir = Path::new(&manifest_dir).parent().unwrap().parent().unwrap();
    let data_dir = root_dir.join("tests").join("data");

    // Ensure test data exists
    if !data_dir.exists() {
        eprintln!("Warning: tests/data directory not found at {:?}", data_dir);
        return;
    }

    // Load setup
    let config = Config::default();
    let rules = get_all_rules(&config, vec![]); // No remote rules for now

    let walker = WalkBuilder::new(&data_dir).build();
    for entry in walker {
        let entry = entry.unwrap();
        if !entry.file_type().unwrap().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("txt") {
            continue;
        }
        if path
            .file_name()
            .and_then(|s| s.to_str())
            .map(|s| s.starts_with('_'))
            .unwrap_or(false)
        {
            continue;
        }

        let content = fs::read_to_string(path).unwrap();

        // Parse Meta-comments
        let rule_id = parse_meta(&content, "RULE");
        let mode = parse_meta(&content, "MODE");

        if let (Some(target_rule_id), Some(mode)) = (rule_id, mode) {
            println!(
                "Testing {} for rule {} (mode: {})",
                path.display(),
                target_rule_id,
                mode
            );
            let findings = scan_content(&content, path, &rules, &config);

            // Filter findings for the target rule
            let target_findings: Vec<_> = findings
                .iter()
                .filter(|f| f.rule_id == target_rule_id)
                .collect();

            if mode == "hit" {
                assert!(
                    !target_findings.is_empty(),
                    "Expected at least 1 finding for rule '{}' in {:?}, but found 0.",
                    target_rule_id,
                    path
                );
            } else if mode == "fp" {
                let match_text: Vec<_> =
                    target_findings.iter().map(|f| &f.matched_content).collect();
                assert!(
                    target_findings.is_empty(),
                    "Expected 0 findings for rule '{}' in {:?} (FP test), but found {}: {:?}",
                    target_rule_id,
                    path,
                    target_findings.len(),
                    match_text
                );
            } else {
                panic!("Unknown mode '{}' in {:?}", mode, path);
            }
        }
    }
}

fn parse_meta(content: &str, key: &str) -> Option<String> {
    for line in content.lines() {
        let prefix = format!("# {}:", key);
        if line.starts_with(&prefix) {
            return Some(line[prefix.len()..].trim().to_string());
        }
    }
    None
}

#[test]
fn fake_slack_tokens_have_expected_format() {
    let bot = fake_slack_bot_token();
    let user = fake_slack_user_token();

    // 期待値も、1本リテラルにせず concat! で作る
    const EXPECTED_BOT: &str = concat!(
        "xoxb-",
        "1234567890",
        "-",
        "1234567890",
        "-",
        "VEILTESTFAKE1234567890VEIL"
    );

    const EXPECTED_USER: &str = concat!(
        "xoxp-",
        "1234567890",
        "-",
        "1234567890",
        "-",
        "VEILTESTFAKE1234567890VEIL"
    );

    assert_eq!(bot, EXPECTED_BOT);
    assert_eq!(user, EXPECTED_USER);
}

#[test]
fn rule_creds_slack_token_legacy_detects_dynamic_tokens() {
    let content = format!(
        "bot={} user={}",
        fake_slack_bot_token(),
        fake_slack_user_token(),
    );

    let config = Config::default();
    let rules = get_all_rules(&config, vec![]);

    // We use a dummy path
    let path = Path::new("memory_test/slack.txt");
    let findings = scan_content(&content, path, &rules, &config);

    assert!(
        findings
            .iter()
            .any(|f| f.rule_id == "creds.slack.token.legacy"),
        "Slack legacy rule did not detect fake tokens"
    );

    let target_matches = findings
        .iter()
        .filter(|f| f.rule_id == "creds.slack.token.legacy")
        .count();
    assert_eq!(target_matches, 2, "Should detect both bot and user tokens");
}
