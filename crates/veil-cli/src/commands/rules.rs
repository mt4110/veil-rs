use crate::config_loader::load_effective_config;
use anyhow::{bail, Result};
use colored::Colorize;
use prettytable::{format, Cell, Row, Table};
use std::path::PathBuf;
use veil_core::{get_all_rules, Rule, Severity};

fn load_rules(config_path: Option<&PathBuf>) -> Result<(veil_config::Config, Vec<Rule>)> {
    let config = load_effective_config(config_path)?;

    let mut remote_rules = Vec::new();
    let remote_url = std::env::var("VEIL_REMOTE_RULES_URL")
        .ok()
        .or_else(|| config.core.remote_rules_url.clone());

    if let Some(url) = remote_url {
        // Timeout 3s is reasonable for helpful commands
        match veil_core::remote::fetch_remote_rules(&url, 3) {
            Ok(rules) => remote_rules = rules,
            Err(e) => eprintln!(
                "Warning: Failed to fetch remote rules from {}: {}. Using local rules only.",
                url, e
            ),
        }
    }

    let rules = get_all_rules(&config, remote_rules);
    Ok((config, rules))
}

pub fn list(config_path: Option<&PathBuf>, severity_filter: Option<Severity>) -> Result<()> {
    let (_config, mut rules) = load_rules(config_path)?;

    // Filter rules if severity is specified
    if let Some(min_sev) = severity_filter {
        rules.retain(|r| r.severity >= min_sev);
    }

    // Sort by ID to make it readable
    rules.sort_by(|a, b| a.id.cmp(&b.id));

    let mut table = Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);

    table.set_titles(Row::new(vec![
        Cell::new("ID").style_spec("b"),
        Cell::new("Severity").style_spec("b"),
        Cell::new("Score").style_spec("b"),
        Cell::new("Category").style_spec("b"),
        Cell::new("Description").style_spec("b"),
    ]));

    for rule in rules {
        table.add_row(Row::new(vec![
            Cell::new(&rule.id),
            Cell::new(&rule.severity.to_string()),
            Cell::new(&rule.score.to_string()),
            Cell::new(&rule.category),
            Cell::new(&rule.description),
        ]));
    }

    table.printstd();

    Ok(())
}

pub fn explain(config_path: Option<&PathBuf>, rule_id: &str) -> Result<()> {
    let (_config, rules) = load_rules(config_path)?;

    // Find the rule
    let Some(rule) = rules.into_iter().find(|r| r.id == rule_id) else {
        bail!("Rule not found: {}", rule_id);
    };

    println!("{}:          {}", "ID".bold(), rule.id);
    println!("{}: {}", "Description".bold(), rule.description);
    println!("{}:    {}", "Severity".bold(), rule.severity);
    println!("{}:       {}", "Score".bold(), rule.score);
    println!("{}:    {}", "Category".bold(), rule.category);

    if !rule.tags.is_empty() {
        println!("{}:        {}", "Tags".bold(), rule.tags.join(", "));
    }

    println!();
    println!("{}", "Pattern:".bold().underline());
    println!("{}", rule.pattern.as_str());

    println!();
    println!("{}", "Context:".bold().underline());
    println!(
        "Before: {} lines / After: {} lines",
        rule.context_lines_before, rule.context_lines_after
    );

    Ok(())
}
