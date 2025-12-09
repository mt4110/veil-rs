use anyhow::Result;
use std::io::{self, BufRead};
use veil_config::MaskMode;
use veil_core::{apply_masks, get_default_rules};

pub fn filter() -> Result<()> {
    // Note: 'filter' currently only does simple rule matching without context scoring heavily
    // because line-by-line piping often lacks global file context.
    // Re-using the same rules as scan.

    let rules = get_default_rules();
    let stdin = io::stdin();

    for line in stdin.lock().lines() {
        let line = line?;
        let mut ranges = Vec::new();

        for rule in &rules {
            // Basic regex finding
            for mat in rule.pattern.find_iter(&line) {
                ranges.push(mat.range());
            }
        }

        // Masking (defaulting to Redact for filter command, maybe configurable later)
        let masked_line = apply_masks(
            &line,
            ranges,
            MaskMode::Redact,
            veil_core::DEFAULT_PLACEHOLDER,
        );
        println!("{}", masked_line);
    }
    Ok(())
}
