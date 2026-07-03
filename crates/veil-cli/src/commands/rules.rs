use crate::cli::TemplatePromoteArgs;
use crate::config_loader::load_effective_config;
use anyhow::{bail, Result};
use colored::Colorize;
use prettytable::{format, Cell, Row, Table};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs;
use std::path::{Component, Path, PathBuf};
use veil_core::{try_get_all_rules, Rule, Severity};

const TEMPLATE_MANIFEST_HEADER: &str = "path,id,concept,slug,variant,category,severity,score,terms";

#[derive(Debug, Clone)]
struct TemplateManifestRow {
    path: PathBuf,
    id: String,
    concept: String,
    slug: String,
    variant: String,
    category: String,
    severity: Severity,
    score: u32,
    terms: String,
}

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

    let rules = try_get_all_rules(&config, remote_rules)?;
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

pub fn promote_templates(args: &TemplatePromoteArgs) -> Result<()> {
    validate_promote_args(args)?;

    let manifest_path = args.templates_dir.join("MANIFEST.csv");
    let rows = read_template_manifest(&manifest_path)?;
    let loaded_rules = veil_core::rules::pack::load_rule_templates_parallel(&args.templates_dir)?;
    let loaded_ids: HashSet<String> = loaded_rules.into_iter().map(|rule| rule.id).collect();

    let selected: Vec<TemplateManifestRow> = rows
        .iter()
        .filter(|row| row_matches_filters(row, args))
        .cloned()
        .collect();

    if selected.is_empty() {
        bail!("No templates matched the requested filters");
    }

    let mut selected_ids = BTreeSet::new();
    for row in &selected {
        if !selected_ids.insert(row.id.clone()) {
            bail!("Duplicate template id in selection: {}", row.id);
        }
        if !loaded_ids.contains(&row.id) {
            bail!(
                "Template manifest references id '{}' but no loaded rule has that id",
                row.id
            );
        }
    }

    if args.dry_run {
        print_promotion_summary("DRY-RUN", &selected);
        return Ok(());
    }

    prepare_output_dir(&args.out_dir, args.force)?;

    let mut manifest_files = Vec::with_capacity(selected.len());
    let mut promoted_rows = Vec::with_capacity(selected.len());

    for row in &selected {
        let source = args.templates_dir.join(&row.path);
        if !source.exists() {
            bail!("Template file listed in manifest is missing: {:?}", source);
        }

        let file_name = row
            .path
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("Template path has no file name: {:?}", row.path))?;
        let output_relative = PathBuf::from("rules")
            .join(&row.category)
            .join(&row.variant)
            .join(file_name);
        let output_path = args.out_dir.join(&output_relative);

        if let Some(parent) = output_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(&source, &output_path)?;

        manifest_files.push(normalize_manifest_path(&output_relative));

        let mut promoted_row = row.clone();
        promoted_row.path = output_relative;
        promoted_rows.push(promoted_row);
    }

    write_rule_pack_manifest(
        &args.out_dir.join("00_manifest.toml"),
        args,
        &manifest_files,
    )?;
    write_template_manifest(&args.out_dir.join("MANIFEST.csv"), &promoted_rows)?;
    print_promotion_summary("PROMOTED", &promoted_rows);
    println!("Output RulePack: {}", args.out_dir.display());

    Ok(())
}

fn validate_promote_args(args: &TemplatePromoteArgs) -> Result<()> {
    if !args.templates_dir.exists() {
        bail!(
            "Template directory does not exist: {}",
            args.templates_dir.display()
        );
    }
    if !args.templates_dir.is_dir() {
        bail!(
            "Template path is not a directory: {}",
            args.templates_dir.display()
        );
    }
    if let (Some(min), Some(max)) = (args.min_score, args.max_score) {
        if min > max {
            bail!("--min-score cannot be greater than --max-score");
        }
    }
    if args.pack_id.is_empty() {
        bail!("--pack-id cannot be empty");
    }
    if !args
        .pack_id
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
    {
        bail!("--pack-id may contain only ASCII letters, digits, '.', '_' and '-'");
    }
    validate_output_dir_is_outside_template_dir(&args.templates_dir, &args.out_dir)?;
    if !args.allow_all
        && args.category.is_empty()
        && args.variant.is_empty()
        && args.severity.is_empty()
        && args.min_score.is_none()
        && args.max_score.is_none()
    {
        bail!("Refusing to promote every template without at least one filter or --allow-all");
    }

    Ok(())
}

fn validate_output_dir_is_outside_template_dir(templates_dir: &Path, out_dir: &Path) -> Result<()> {
    let templates_dir = templates_dir.canonicalize()?;
    let out_dir = absolute_path_for_maybe_missing(out_dir)?;

    if out_dir.starts_with(&templates_dir) {
        bail!("--out-dir must be outside --templates-dir to keep inactive templates separate from executable RulePacks");
    }

    Ok(())
}

fn absolute_path_for_maybe_missing(path: &Path) -> Result<PathBuf> {
    if path.exists() {
        return path.canonicalize().map_err(Into::into);
    }

    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .ok_or_else(|| anyhow::anyhow!("Output directory has no final path segment"))?;

    Ok(parent.canonicalize()?.join(file_name))
}

fn read_template_manifest(path: &Path) -> Result<Vec<TemplateManifestRow>> {
    let content = fs::read_to_string(path)
        .map_err(|err| anyhow::anyhow!("Failed to read {}: {err}", path.display()))?;
    let mut lines = content.lines();
    let header = lines
        .next()
        .ok_or_else(|| anyhow::anyhow!("Template manifest is empty: {}", path.display()))?
        .trim_end_matches('\r');

    if header != TEMPLATE_MANIFEST_HEADER {
        bail!(
            "Unexpected template manifest header in {}: expected '{}'",
            path.display(),
            TEMPLATE_MANIFEST_HEADER
        );
    }

    let mut rows = Vec::new();
    for (index, line) in lines.enumerate() {
        let line_no = index + 2;
        let line = line.trim_end_matches('\r');
        if line.trim().is_empty() {
            continue;
        }

        let cells = split_manifest_csv_line(line)
            .map_err(|err| anyhow::anyhow!("Invalid MANIFEST.csv line {line_no}: {err}"))?;
        if cells.len() != 9 {
            bail!(
                "Invalid MANIFEST.csv line {}: expected 9 columns, got {}",
                line_no,
                cells.len()
            );
        }

        let score = cells[7]
            .parse::<u32>()
            .map_err(|err| anyhow::anyhow!("Invalid score on line {line_no}: {err}"))?;
        let severity = parse_manifest_severity(&cells[6])
            .map_err(|err| anyhow::anyhow!("Invalid severity on line {line_no}: {err}"))?;

        let row = TemplateManifestRow {
            path: PathBuf::from(&cells[0]),
            id: cells[1].clone(),
            concept: cells[2].clone(),
            slug: cells[3].clone(),
            variant: cells[4].clone(),
            category: cells[5].clone(),
            severity,
            score,
            terms: cells[8].clone(),
        };
        validate_manifest_row(&row, line_no)?;
        rows.push(row);
    }

    if rows.is_empty() {
        bail!("Template manifest has no rows: {}", path.display());
    }

    Ok(rows)
}

fn validate_manifest_row(row: &TemplateManifestRow, line_no: usize) -> Result<()> {
    if row.id.trim().is_empty() {
        bail!("Invalid MANIFEST.csv line {}: id is empty", line_no);
    }
    if !is_safe_relative_path(&row.path) {
        bail!(
            "Invalid MANIFEST.csv line {}: path must be a safe relative path",
            line_no
        );
    }
    if row.path.extension().and_then(|ext| ext.to_str()) != Some("toml") {
        bail!(
            "Invalid MANIFEST.csv line {}: path must end in .toml",
            line_no
        );
    }
    if !is_safe_segment(&row.category) {
        bail!(
            "Invalid MANIFEST.csv line {}: category must be a safe path segment",
            line_no
        );
    }
    if !is_safe_segment(&row.variant) {
        bail!(
            "Invalid MANIFEST.csv line {}: variant must be a safe path segment",
            line_no
        );
    }
    if row.score > 100 {
        bail!(
            "Invalid MANIFEST.csv line {}: score must be <= 100",
            line_no
        );
    }

    Ok(())
}

fn parse_manifest_severity(value: &str) -> Result<Severity> {
    match value.to_ascii_lowercase().as_str() {
        "low" => Ok(Severity::Low),
        "medium" => Ok(Severity::Medium),
        "high" => Ok(Severity::High),
        "critical" => Ok(Severity::Critical),
        _ => bail!("expected low, medium, high, or critical"),
    }
}

fn split_manifest_csv_line(line: &str) -> Result<Vec<String>> {
    let mut cells = Vec::new();
    let mut cell = String::new();
    let mut chars = line.chars().peekable();
    let mut in_quotes = false;

    while let Some(ch) = chars.next() {
        if in_quotes {
            if ch == '"' {
                if chars.peek() == Some(&'"') {
                    cell.push('"');
                    chars.next();
                } else {
                    in_quotes = false;
                }
            } else {
                cell.push(ch);
            }
            continue;
        }

        match ch {
            ',' => {
                cells.push(cell);
                cell = String::new();
            }
            '"' if cell.is_empty() => in_quotes = true,
            _ => cell.push(ch),
        }
    }

    if in_quotes {
        bail!("unterminated quoted field");
    }

    cells.push(cell);
    Ok(cells)
}

fn is_safe_relative_path(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
}

fn is_safe_segment(segment: &str) -> bool {
    !segment.is_empty()
        && segment
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-'))
}

fn row_matches_filters(row: &TemplateManifestRow, args: &TemplatePromoteArgs) -> bool {
    (args.category.is_empty()
        || args
            .category
            .iter()
            .any(|category| category == &row.category))
        && (args.variant.is_empty() || args.variant.iter().any(|variant| variant == &row.variant))
        && (args.severity.is_empty()
            || args
                .severity
                .iter()
                .any(|severity| severity == &row.severity))
        && args.min_score.is_none_or(|min| row.score >= min)
        && args.max_score.is_none_or(|max| row.score <= max)
}

fn prepare_output_dir(out_dir: &Path, force: bool) -> Result<()> {
    if out_dir.exists() && !out_dir.is_dir() {
        bail!(
            "Output path exists but is not a directory: {}",
            out_dir.display()
        );
    }
    if out_dir.exists() && !force && out_dir.read_dir()?.next().is_some() {
        bail!(
            "Output directory is not empty: {}. Pass --force to update generated files.",
            out_dir.display()
        );
    }
    fs::create_dir_all(out_dir)?;
    Ok(())
}

fn write_rule_pack_manifest(
    path: &Path,
    args: &TemplatePromoteArgs,
    files: &[String],
) -> Result<()> {
    let mut manifest = String::new();
    manifest.push_str("files = [\n");
    for file in files {
        manifest.push_str(&format!("  {},\n", toml_string(file)));
    }
    manifest.push_str("]\n\n");
    manifest.push_str("[pack]\n");
    manifest.push_str(&format!("id = {}\n", toml_string(&args.pack_id)));
    manifest.push_str("version = 1\n");
    manifest.push_str("schema_version = 1\n");
    manifest.push_str(&format!(
        "description = {}\n",
        toml_string("Promoted JP security template RulePack")
    ));

    fs::write(path, manifest)?;
    Ok(())
}

fn write_template_manifest(path: &Path, rows: &[TemplateManifestRow]) -> Result<()> {
    let mut content = String::new();
    content.push_str(TEMPLATE_MANIFEST_HEADER);
    content.push('\n');

    for row in rows {
        let cells = [
            normalize_manifest_path(&row.path),
            row.id.clone(),
            row.concept.clone(),
            row.slug.clone(),
            row.variant.clone(),
            row.category.clone(),
            row.severity.to_string().to_ascii_lowercase(),
            row.score.to_string(),
            row.terms.clone(),
        ];
        content.push_str(
            &cells
                .iter()
                .map(|cell| csv_cell(cell))
                .collect::<Vec<_>>()
                .join(","),
        );
        content.push('\n');
    }

    fs::write(path, content)?;
    Ok(())
}

fn normalize_manifest_path(path: &Path) -> String {
    path.components()
        .filter_map(|component| match component {
            Component::Normal(segment) => Some(segment.to_string_lossy().into_owned()),
            Component::CurDir => None,
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn toml_string(value: &str) -> String {
    format!(
        "\"{}\"",
        value
            .replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
    )
}

fn csv_cell(value: &str) -> String {
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

fn print_promotion_summary(label: &str, rows: &[TemplateManifestRow]) {
    println!("{label}: {} templates", rows.len());
    print_counts("category", rows.iter().map(|row| row.category.as_str()));
    print_counts("variant", rows.iter().map(|row| row.variant.as_str()));
    print_counts(
        "severity",
        rows.iter().map(|row| match row.severity {
            Severity::Low => "low",
            Severity::Medium => "medium",
            Severity::High => "high",
            Severity::Critical => "critical",
        }),
    );
}

fn print_counts<'a>(label: &str, values: impl Iterator<Item = &'a str>) {
    let mut counts = BTreeMap::new();
    for value in values {
        *counts.entry(value).or_insert(0usize) += 1;
    }

    let summary = counts
        .into_iter()
        .map(|(value, count)| format!("{value}={count}"))
        .collect::<Vec<_>>()
        .join(", ");
    println!("  {label}: {summary}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn write_template(root: &Path, relative: &str, id: &str, category: &str, score: u32) {
        let path = root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let mut file = fs::File::create(path).unwrap();
        writeln!(
            file,
            r#"[[rules]]
id = "{id}"
description = "{id}"
pattern = "{id}"
severity = "critical"
score = {score}
category = "{category}"
"#
        )
        .unwrap();
    }

    fn template_root() -> TempDir {
        let temp = TempDir::new().unwrap();
        let root = temp.path();
        fs::create_dir_all(root.join("templates/finance/kv")).unwrap();
        fs::create_dir_all(root.join("templates/secret/leak")).unwrap();

        write_template(
            root,
            "templates/finance/kv/log.jp.finance.card.kv.toml",
            "log.jp.finance.card.kv",
            "finance",
            95,
        );
        write_template(
            root,
            "templates/secret/leak/log.jp.secret.token.leak.toml",
            "log.jp.secret.token.leak",
            "secret",
            90,
        );

        fs::write(
            root.join("MANIFEST.csv"),
            "path,id,concept,slug,variant,category,severity,score,terms\n\
templates/finance/kv/log.jp.finance.card.kv.toml,log.jp.finance.card.kv,カード,card,kv,finance,critical,95,カード|card\n\
templates/secret/leak/log.jp.secret.token.leak.toml,log.jp.secret.token.leak,トークン,token,leak,secret,critical,90,トークン|token\n",
        )
        .unwrap();

        temp
    }

    fn template_args(root: &Path, out_dir: &Path) -> TemplatePromoteArgs {
        TemplatePromoteArgs {
            templates_dir: root.to_path_buf(),
            out_dir: out_dir.to_path_buf(),
            category: vec!["finance".to_string()],
            variant: vec!["kv".to_string()],
            severity: vec![Severity::Critical],
            min_score: Some(95),
            max_score: None,
            pack_id: "veil.jp.security.promoted".to_string(),
            dry_run: false,
            force: false,
            allow_all: false,
        }
    }

    #[test]
    fn promote_templates_filters_and_writes_rule_pack() {
        let templates = template_root();
        let output = TempDir::new().unwrap();
        let out_dir = output.path().join("promoted");
        let args = template_args(templates.path(), &out_dir);

        promote_templates(&args).unwrap();

        assert!(out_dir.join("00_manifest.toml").exists());
        assert!(out_dir.join("MANIFEST.csv").exists());
        assert!(out_dir
            .join("rules/finance/kv/log.jp.finance.card.kv.toml")
            .exists());
        assert!(!out_dir
            .join("rules/secret/leak/log.jp.secret.token.leak.toml")
            .exists());

        let rules = veil_core::rules::pack::load_rule_pack(&out_dir).unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].id, "log.jp.finance.card.kv");
    }

    #[test]
    fn promote_templates_dry_run_does_not_write() {
        let templates = template_root();
        let output = TempDir::new().unwrap();
        let out_dir = output.path().join("promoted");
        let mut args = template_args(templates.path(), &out_dir);
        args.dry_run = true;

        promote_templates(&args).unwrap();

        assert!(!out_dir.exists());
    }

    #[test]
    fn promote_templates_requires_filter_or_allow_all() {
        let templates = template_root();
        let output = TempDir::new().unwrap();
        let out_dir = output.path().join("promoted");
        let mut args = template_args(templates.path(), &out_dir);
        args.category.clear();
        args.variant.clear();
        args.severity.clear();
        args.min_score = None;

        let err = promote_templates(&args).unwrap_err();
        assert!(err
            .to_string()
            .contains("Refusing to promote every template"));
    }

    #[test]
    fn promote_templates_rejects_output_inside_template_root() {
        let templates = template_root();
        let out_dir = templates.path().join("promoted");
        let args = template_args(templates.path(), &out_dir);

        let err = promote_templates(&args).unwrap_err();
        assert!(err.to_string().contains("--out-dir must be outside"));
    }
}
