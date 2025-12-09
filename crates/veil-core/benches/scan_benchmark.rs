use divan::Bencher;
use regex::Regex;
use std::path::PathBuf;
use veil_config::Config;
use veil_core::model::Rule;
use veil_core::scanner::{scan_content, utils::scan_data};

fn main() {
    divan::main();
}

// Fixtures
fn get_rules() -> Vec<Rule> {
    vec![
        Rule {
            id: "AWS_ACCESS_KEY".to_string(),
            pattern: Regex::new(
                r"(A3T[A-Z0-9]|AKIA|AGPA|AIDA|AROA|AIPA|ANPA|ANVA|ASIA)[A-Z0-9]{16}",
            )
            .unwrap(),
            description: "AWS Access Key".to_string(),
            severity: veil_core::model::Severity::High,
            score: 100,
            base_score: None,
            category: "Cloud Provider".to_string(),
            tags: vec![],
            validator: None, // No validator for bench to isolate regex perf
            context_lines_before: 2,
            context_lines_after: 0,
        },
        Rule {
            id: "Generic_API_Key".to_string(),
            pattern: Regex::new(
                r#"(?i)(api[_-]?key|apikey)['"]?\s*[:=]\s*['"]?[a-zA-Z0-9]{32}['"]?"#,
            )
            .unwrap(),
            description: "Generic API Key".to_string(),
            severity: veil_core::model::Severity::Medium,
            score: 50,
            base_score: None,
            category: "Generic".to_string(),
            tags: vec![],
            validator: None,
            context_lines_before: 2,
            context_lines_after: 0,
        },
    ]
}

fn get_config() -> Config {
    Config::default()
}

// Benchmarks

#[divan::bench]
fn scan_content_small(bencher: Bencher) {
    let rules = get_rules();
    let config = get_config();
    let content = r#"
        # This is a small config file
        aws_access_key_id = AKIA1234567890ABCDEF
        api_key = "12345678901234567890123456789012"
        # secret = "hidden"
    "#;
    let path = PathBuf::from("test.txt");

    bencher.bench(|| scan_content(divan::black_box(content), &path, &rules, &config));
}

#[divan::bench]
fn scan_content_medium(bencher: Bencher) {
    let rules = get_rules();
    let config = get_config();
    // Generate ~1MB content
    let line = "var x = 'nothing specific here';\n";
    let mut content = String::with_capacity(1024 * 1024);
    for _ in 0..10000 {
        content.push_str(line);
    }
    // Inject some secrets
    content.push_str("aws_access_key_id = AKIA1234567890ABCDEF\n");
    content.push_str("api_key = \"12345678901234567890123456789012\"\n");

    let path = PathBuf::from("medium.txt");

    bencher.bench(|| scan_content(divan::black_box(&content), &path, &rules, &config));
}

#[divan::bench]
fn scan_content_large(bencher: Bencher) {
    let rules = get_rules();
    let config = get_config();
    // Generate ~5MB content
    let line = "log.info('processing request');\n";
    let mut content = String::with_capacity(5 * 1024 * 1024);
    for _ in 0..160000 {
        content.push_str(line);
    }
    // Inject secrets
    content.push_str("aws_access_key_id = AKIA1234567890ABCDEF\n");

    let path = PathBuf::from("large.txt");

    bencher.bench(|| scan_content(divan::black_box(&content), &path, &rules, &config));
}

#[divan::bench]
fn apply_masks_stress(bencher: Bencher) {
    use veil_config::MaskMode;
    use veil_core::masking::apply_masks;

    // Create a line with many overlaps/secrets
    let mut content = String::new();
    let mut ranges = Vec::new();
    let secret = "SECRET1234";

    for _ in 0..100 {
        let start = content.len();
        content.push_str(secret);
        content.push(' ');
        ranges.push(start..start + secret.len());
    }

    bencher.bench(|| {
        apply_masks(
            divan::black_box(&content),
            ranges.clone(),
            MaskMode::Partial,
            veil_core::DEFAULT_PLACEHOLDER,
        )
    });
}

#[divan::bench]
fn scan_data_binary_skip(bencher: Bencher) {
    let rules = get_rules();
    let config = get_config();
    // 5MB binary data (some zeroes)
    let mut data = vec![b'A'; 5 * 1024 * 1024];
    data[100] = 0; // early zero
    let path = PathBuf::from("binary.bin");

    bencher.bench(|| {
        scan_data(
            divan::black_box(path.as_path()),
            divan::black_box(&data),
            &rules,
            &config,
        )
    });
}

#[divan::bench]
fn scan_data_size_skip(bencher: Bencher) {
    let rules = get_rules();
    let config = get_config();
    // 10MB data (over 1MB default limit)
    let data = vec![b'A'; 10 * 1024 * 1024];
    let path = PathBuf::from("large_skip.txt");

    bencher.bench(|| {
        scan_data(
            divan::black_box(path.as_path()),
            divan::black_box(&data),
            &rules,
            &config,
        )
    });
}
