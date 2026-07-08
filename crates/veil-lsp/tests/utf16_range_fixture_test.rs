use std::fs;
use std::path::{Path, PathBuf};

use tower_lsp::lsp_types::NumberOrString;
use veil_config::Config;
use veil_core::try_get_all_rules;
use veil_lsp::server::diagnostics_for_text;

#[test]
fn lsp_diagnostics_follow_utf16_range_fixtures() {
    let config = Config::default();
    let rules = try_get_all_rules(&config, Vec::new()).expect("rules");

    for fixture_path in fixture_paths() {
        let (expectation, content) = load_fixture(&fixture_path);
        let diagnostics = diagnostics_for_text(&content, &fixture_path, &rules, &config);

        assert_eq!(
            diagnostics.len(),
            1,
            "fixture should produce exactly one diagnostic: {}",
            fixture_path.display()
        );

        let diagnostic = diagnostics
            .iter()
            .find(|diagnostic| {
                matches!(
                    diagnostic.code.as_ref(),
                    Some(NumberOrString::String(code)) if code == &expectation.rule_id
                )
            })
            .unwrap_or_else(|| {
                panic!(
                    "expected diagnostic '{}' in {}",
                    expectation.rule_id,
                    fixture_path.display()
                )
            });

        assert_eq!(
            diagnostic.range.start.line,
            expectation.line,
            "unexpected line for {}",
            fixture_path.display()
        );
        assert_eq!(
            diagnostic.range.start.character,
            expectation.start_character,
            "unexpected start character for {}",
            fixture_path.display()
        );
        assert_eq!(
            diagnostic.range.end.line,
            expectation.line,
            "unexpected end line for {}",
            fixture_path.display()
        );
        assert_eq!(
            diagnostic.range.end.character,
            expectation.end_character,
            "unexpected end character for {}",
            fixture_path.display()
        );

        let data = diagnostic.data.as_ref().expect("diagnostic data");
        let data_text = data.to_string();
        assert!(
            !data_text.contains(&expectation.forbidden_data),
            "diagnostic data leaked raw match for {}",
            fixture_path.display()
        );
        assert!(
            !data_text.contains(content.trim()),
            "diagnostic data leaked raw fixture body for {}",
            fixture_path.display()
        );
    }
}

#[derive(Debug, PartialEq, Eq)]
struct FixtureExpectation {
    rule_id: String,
    line: u32,
    start_character: u32,
    end_character: u32,
    forbidden_data: String,
}

fn fixture_paths() -> Vec<PathBuf> {
    let fixture_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("utf16");
    let mut paths: Vec<_> = fs::read_dir(&fixture_dir)
        .expect("fixture dir")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("txt"))
        .collect();
    paths.sort();
    paths
}

fn load_fixture(path: &Path) -> (FixtureExpectation, String) {
    let fixture = fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("failed to read fixture {}: {error}", path.display()));
    let mut rule_id = None;
    let mut line = None;
    let mut start_character = None;
    let mut end_character = None;
    let mut forbidden_data = None;
    let mut body_lines = Vec::new();
    let mut in_body = false;

    for raw_line in fixture.lines() {
        if in_body {
            body_lines.push(raw_line);
            continue;
        }

        if raw_line.trim().is_empty() {
            in_body = true;
            continue;
        }

        if let Some(value) = raw_line.strip_prefix("# EXPECT_RULE:") {
            rule_id = Some(value.trim().to_string());
            continue;
        }
        if let Some(value) = raw_line.strip_prefix("# EXPECT_LINE:") {
            line = Some(value.trim().parse::<u32>().expect("EXPECT_LINE"));
            continue;
        }
        if let Some(value) = raw_line.strip_prefix("# EXPECT_START:") {
            start_character = Some(value.trim().parse::<u32>().expect("EXPECT_START"));
            continue;
        }
        if let Some(value) = raw_line.strip_prefix("# EXPECT_END:") {
            end_character = Some(value.trim().parse::<u32>().expect("EXPECT_END"));
            continue;
        }
        if let Some(value) = raw_line.strip_prefix("# FORBID_DATA:") {
            forbidden_data = Some(value.trim().to_string());
            continue;
        }

        panic!(
            "unsupported fixture metadata line in {}: {}",
            path.display(),
            raw_line
        );
    }

    (
        FixtureExpectation {
            rule_id: rule_id.expect("EXPECT_RULE"),
            line: line.expect("EXPECT_LINE"),
            start_character: start_character.expect("EXPECT_START"),
            end_character: end_character.expect("EXPECT_END"),
            forbidden_data: forbidden_data.expect("FORBID_DATA"),
        },
        body_lines.join("\n"),
    )
}
