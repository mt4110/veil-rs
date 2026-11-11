#![no_main]

use libfuzzer_sys::fuzz_target;
use std::collections::HashSet;
use veil_core::rules::pack::{load_rules_from_content, parse_manifest};

fuzz_target!(|data: &[u8]| {
    // The production loader rejects invalid UTF-8 before parsing TOML.
    if let Ok(content) = std::str::from_utf8(data) {
        let _ = parse_manifest(content);
        let _ = load_rules_from_content(content, &mut Vec::new(), &mut HashSet::new(), None);
    }
});
