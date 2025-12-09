use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use veil_config::{load_config, Config};
use veil_core::{get_default_rules, scan_path};

pub fn mask(
    paths: &[PathBuf],
    config_path: Option<&PathBuf>,
    dry_run: bool,
    backup_suffix: Option<String>,
) -> Result<()> {
    let config_file = config_path
        .cloned()
        .unwrap_or_else(|| PathBuf::from("veil.toml"));
    let config = load_config(&config_file).unwrap_or_else(|_| Config::default());
    let rules = get_default_rules();

    for path in paths {
        // Collect findings first to avoid reading/writing overlap
        // scan_path gives us Finding which contains masked_line, but not the whole file content structure.
        // We need to rewrite the file.

        // MVP: Read matching files, apply substitution line by line, write back.
        // For efficiency, we only rewrite if findings exist.

        let mut findings = scan_path(path, &rules, &config);

        if findings.is_empty() {
            continue;
        }

        // Sort findings by line number
        findings.sort_by_key(|f| f.line_number);

        // Read file again
        let content = fs::read_to_string(path)?;
        let mut new_lines = Vec::new();
        let mut masked_count = 0;

        for (i, line) in content.lines().enumerate() {
            let line_num = i + 1;
            // Check if this line has a finding
            if let Some(finding) = findings.iter().find(|f| f.line_number == line_num) {
                new_lines.push(finding.masked_snippet.clone());
                masked_count += 1;
            } else {
                new_lines.push(line.to_string());
            }
        }

        // Add newline at end if original had? simple join for now
        let new_content = new_lines.join("\n");

        if dry_run {
            println!(
                "[DRY-RUN] Would mask {} secrets in: {}",
                masked_count,
                path.display()
            );
            // Optionally print preview?
        } else {
            // Backup
            if let Some(suffix) = &backup_suffix {
                let mut backup_path = path.clone();
                if let Some(ext) = path.extension() {
                    let mut new_ext = ext.to_os_string();
                    new_ext.push(suffix);
                    backup_path.set_extension(new_ext);
                } else {
                    backup_path.set_extension(suffix.trim_start_matches('.'));
                }

                // Simpler backup naming: path + suffix
                // e.g. file.txt + .bak -> file.txt.bak
                // The above logic replaces extension which might be wrong.
                // Let's use simple string concatenation:
                let s = path.to_string_lossy();
                let backup_path = PathBuf::from(format!("{}{}", s, suffix));

                fs::write(&backup_path, &content)?;
                println!("Created backup: {}", backup_path.display());
            }

            fs::write(path, new_content)?;
            println!("Masked {} secrets in: {}", masked_count, path.display());
        }
    }

    Ok(())
}
