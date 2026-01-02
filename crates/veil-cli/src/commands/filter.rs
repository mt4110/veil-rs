use crate::config_loader::load_effective_config;
use anyhow::Result;
use std::io::{self, BufRead};
use std::path::PathBuf;
use veil_config::MaskMode;
use veil_core::{apply_masks_spans, get_all_rules, MaskSpan};

pub fn filter(config_path: Option<&PathBuf>) -> Result<()> {
    // Load effective config (layered)
    let config = load_effective_config(config_path)?;

    // Use unified rules (config + empty internal rules for now)
    let rules = get_all_rules(&config, vec![]);

    // Apply masking config
    let mask_mode = config.output.mask_mode.unwrap_or(MaskMode::Redact);
    let global_placeholder = config.masking.placeholder;

    let stdin = io::stdin();

    for line in stdin.lock().lines() {
        let line = line?;
        let mut spans = Vec::new();

        for rule in &rules {
            // Determine priority
            let base_priority = match rule.category.as_str() {
                "secret" => 100,
                "pii" => 50,
                "observability" => 20,
                _ => 10,
            };
            // Operational tie-breaker: Slight boost for Log Pack rules to prefer them over generic rules
            // in case of equivalent scores (Same Category/Severity).
            let priority = if rule.id.starts_with("log.") {
                base_priority + 5
            } else {
                base_priority
            };
            // DEBUG
            // eprintln!("DEBUG: Rule: {}, Placeholder: {:?}", rule.id, rule.placeholder);

            // Determine placeholder
            // Rule > Config > Default (Config usually has default)
            let ph = rule
                .placeholder
                .as_ref()
                .unwrap_or(&global_placeholder)
                .clone();

            for mat in rule.pattern.find_iter(&line) {
                spans.push(MaskSpan {
                    start: mat.start(),
                    end: mat.end(),
                    placeholder: ph.clone(),
                    priority,
                });
            }
        }

        // Masking
        let masked_line = apply_masks_spans(&line, spans, mask_mode.clone());
        println!("{}", masked_line);
    }
    Ok(())
}
