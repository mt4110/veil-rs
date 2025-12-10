use std::time::Instant;
use veil_config::Config;
use veil_core::{model::Severity, scan_content, Rule};

#[test]
fn test_large_input_performance() {
    // Generate 5MB of repeated data
    // Pattern "SECRET" triggers a match. "AAAA..." is noise.
    let repeat_count = 5_000_000;
    let mut content = String::with_capacity(repeat_count + 100);
    content.push_str(&"A".repeat(repeat_count));
    content.push_str("SECRET_KEY=12345");
    content.push_str(&"B".repeat(100));

    let rule = Rule {
        id: "test_large".to_string(),
        pattern: regex::Regex::new("SECRET_KEY=\\d+").unwrap(),
        description: "test".to_string(),
        severity: Severity::High,
        score: 50,
        category: "test".to_string(),
        tags: vec![],
        base_score: None,
        context_lines_before: 2,
        context_lines_after: 0,
        validator: None,
    };
    let rules = vec![rule];
    let config = Config::default();

    let start = Instant::now();
    let findings = scan_content(&content, std::path::Path::new("dummy"), &rules, &config);
    let duration = start.elapsed();

    // Verification
    assert_eq!(findings.len(), 1, "Should find the secret at the end");
    assert!(
        duration.as_millis() < 2000,
        "Scanning 5MB should be under 2s (took {}ms)",
        duration.as_millis()
    );
}

#[test]
fn test_many_matches_dos() {
    // Generate input with MANY matches (100k matches)
    // To ensure accumulating findings doesn't explode memory or time unreasonably
    let mut content = String::new();
    for _ in 0..10_000 {
        content.push_str("AWS_ACCESS_KEY_ID=AKIAIOSFODNN7EXAMPLE\n");
    }

    let rule = Rule {
        id: "aws".to_string(),
        pattern: regex::Regex::new("AWS_ACCESS_KEY_ID=AKIA[0-9A-Z]{16}").unwrap(),
        description: "aws".to_string(),
        severity: Severity::High,
        score: 50,
        category: "creds".to_string(),
        tags: vec![],
        base_score: None,
        context_lines_before: 0,
        context_lines_after: 0,
        validator: None,
    };
    let rules = vec![rule];
    let config = Config::default();

    let start = Instant::now();
    let findings = scan_content(&content, std::path::Path::new("dummy"), &rules, &config);
    let duration = start.elapsed();

    assert_eq!(findings.len(), 10_000);
    assert!(
        duration.as_millis() < 3000,
        "10k matches should be fast (took {}ms)",
        duration.as_millis()
    );
}
