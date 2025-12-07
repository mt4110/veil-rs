use anyhow::Result;
use std::fs;
use std::path::PathBuf;
use veil_config::{load_config, Config};
use veil_core::{get_default_rules, scan_path};

pub fn mask(paths: &[PathBuf], config_path: Option<&PathBuf>) -> Result<()> {
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

        for (i, line) in content.lines().enumerate() {
            let line_num = i + 1;
            // Check if this line has a finding
            if let Some(finding) = findings.iter().find(|f| f.line_number == line_num) {
                new_lines.push(finding.masked_line.clone());
            } else {
                new_lines.push(line.to_string());
            }
        }

        // Add newline at end if original had? simple join for now
        let new_content = new_lines.join("\n");

        // Create backup
        // let backup_path = path.with_extension(format!("{}.bk", path.extension().unwrap_or_default().to_string_lossy()));
        // fs::write(&backup_path, &content)?;

        fs::write(path, new_content)?;
        println!("Masked secrets in: {}", path.display());
    }

    Ok(())
}
