use anyhow::Result;
use std::io::{self, BufRead};
use veil_core::{get_default_rules, mask_ranges};

pub fn filter() -> Result<()> {
    // Note: 'filter' currently only does simple rule matching without context scoring heavily
    // because line-by-line piping often lacks global file context.
    // Re-using the same rules as scan.

    let rules = get_default_rules();
    let stdin = io::stdin();

    for line_res in stdin.lock().lines() {
        let line = line_res?;

        let mut ranges = Vec::new();
        for rule in &rules {
            for mat in rule.pattern.find_iter(&line) {
                ranges.push(mat.range());
            }
        }

        // mask_ranges handles overlaps and multiple occurrences
        let masked = mask_ranges(&line, ranges);
        println!("{}", masked);
    }

    Ok(())
}
