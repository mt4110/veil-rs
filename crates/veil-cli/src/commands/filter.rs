use anyhow::Result;
use std::io::{self, BufRead};
use veil_core::{get_default_rules, mask_string};

pub fn filter() -> Result<()> {
    // Note: 'filter' currently only does simple rule matching without context scoring heavily
    // because line-by-line piping often lacks global file context.
    // Re-using the same rules as scan.

    let rules = get_default_rules();
    let stdin = io::stdin();

    for line_res in stdin.lock().lines() {
        let line = line_res?;
        let mut masked = line.clone();

        for rule in &rules {
            if let Some(mat) = rule.pattern.find(&line) {
                masked = mask_string(&line, mat.range());
                // If multiple rules match, we should continue masking the already masked string?
                // Simple approach: Only one mask per line per rule pass.
                // Ideally we loop until no more matches or be careful with indices.
                // For MVP: simple fallback.
            }
        }

        println!("{}", masked);
    }

    Ok(())
}
