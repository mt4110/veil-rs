use anyhow::{bail, Context, Result};
use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use veil_core::scanner::result::FileScanStatus;

struct PreparedMask {
    display_path: PathBuf,
    path: PathBuf,
    original: Vec<u8>,
    replacement: String,
    findings: usize,
    permissions: fs::Permissions,
    backup: Option<PathBuf>,
}

pub fn mask(
    paths: &[PathBuf],
    config_path: Option<&PathBuf>,
    dry_run: bool,
    backup_suffix: Option<String>,
) -> Result<()> {
    let config = crate::config_loader::load_effective_config(config_path)?;
    let rules = veil_core::try_get_all_rules(&config, Vec::new())?;
    if paths.is_empty() {
        bail!("mask requires at least one regular file");
    }
    if let Some(suffix) = &backup_suffix {
        if suffix.is_empty() || suffix.contains('/') || suffix.contains('\\') {
            bail!("backup suffix must be nonempty and contain no path separators");
        }
    }

    // Resolve and validate all destinations before preparing any writes.
    let mut input_paths = HashSet::new();
    let mut inputs = Vec::new();
    for path in paths {
        require_regular_file(path)?;
        let canonical = fs::canonicalize(path)?;
        if !input_paths.insert(canonical.clone()) {
            bail!("duplicate mask input: {}", path.display());
        }
        inputs.push((path, canonical));
    }
    inputs.retain(|(display_path, _)| {
        !config
            .core
            .ignore
            .iter()
            .any(|pattern| display_path.to_string_lossy().contains(pattern))
    });
    if inputs.len()
        > config
            .core
            .max_file_count
            .unwrap_or(veil_core::DEFAULT_MAX_FILE_COUNT)
    {
        bail!("Cannot mask: scan incomplete: file count limit reached (core.max_file_count)");
    }

    let mut prepared = Vec::new();
    let mut total_findings = 0;
    for (display_path, path) in inputs {
        let backup = backup_suffix.as_ref().map(|suffix| {
            let mut name = path.as_os_str().to_os_string();
            name.push(suffix);
            PathBuf::from(name)
        });
        if let Some(backup) = &backup {
            if input_paths.contains(backup) {
                bail!(
                    "backup destination is another mask input: {}",
                    backup.display()
                );
            }
            require_absent(backup)?;
        }

        let mut file = fs::File::open(&path)?;
        let metadata = file.metadata()?;
        let mut header = [0; 8192];
        let count = file.read(&mut header)?;
        if header[..count].contains(&0) {
            continue; // Binary assets are intentional exclusions, as in scan.
        }
        let max_size = config
            .core
            .max_file_size
            .unwrap_or(veil_core::DEFAULT_MAX_FILE_SIZE_BYTES);
        if metadata.len() > max_size {
            bail!("Cannot mask: scan incomplete: file size limit reached (core.max_file_size)");
        }
        file.seek(SeekFrom::Start(0))?;
        let mut original = Vec::new();
        file.take(max_size + 1).read_to_end(&mut original)?;
        let findings = veil_core::scan_data(&path, &original, &rules, &config);
        match FileScanStatus::from_findings(&findings) {
            FileScanStatus::Binary => continue,
            FileScanStatus::TooLarge => {
                bail!("Cannot mask: scan incomplete: file size limit reached (core.max_file_size)")
            }
            FileScanStatus::ReadError => {
                bail!("Cannot mask: scan incomplete: file read or UTF-8 decoding error")
            }
            FileScanStatus::Scanned => {}
        }
        total_findings += findings.len();
        if config
            .output
            .max_findings
            .is_some_and(|max| total_findings >= max)
        {
            bail!("Cannot mask: scan incomplete: finding limit reached (output.max_findings)");
        }
        if findings.is_empty() {
            continue;
        }
        if metadata.permissions().readonly() {
            bail!(
                "refusing to mask read-only file: {}",
                display_path.display()
            );
        }
        let content = std::str::from_utf8(&original)?;
        let replacement = masked_content(content, &findings)?;
        prepared.push(PreparedMask {
            display_path: display_path.clone(),
            path,
            original,
            replacement,
            findings: findings.len(),
            permissions: metadata.permissions(),
            backup,
        });
    }

    // No input or backup is written until every input has passed preflight.
    for edit in prepared {
        if dry_run {
            println!(
                "[DRY-RUN] Would mask {} secrets in: {}",
                edit.findings,
                edit.display_path.display()
            );
            continue;
        }
        ensure_unchanged(&edit)?;
        if let Some(backup) = &edit.backup {
            let tmp = write_temporary(backup, &edit.original, edit.permissions.clone())?;
            tmp.persist_noclobber(backup)
                .map_err(|error| error.error)
                .with_context(|| {
                    format!("create backup without overwriting {}", backup.display())
                })?;
            println!("Created backup: {}", backup.display());
        }
        let tmp = write_temporary(
            &edit.path,
            edit.replacement.as_bytes(),
            edit.permissions.clone(),
        )?;
        ensure_unchanged(&edit)?;
        tmp.persist(&edit.path)
            .map_err(|error| error.error)
            .with_context(|| format!("replace {}", edit.path.display()))?;
        println!(
            "Masked {} secrets in: {}",
            edit.findings,
            edit.display_path.display()
        );
    }
    Ok(())
}

fn require_regular_file(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path)
        .with_context(|| format!("Cannot access mask input: {}", path.display()))?;
    if !metadata.file_type().is_file() {
        bail!(
            "mask input must be a regular file, not a directory or symlink: {}",
            path.display()
        );
    }
    Ok(())
}

fn require_absent(path: &Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error).with_context(|| format!("inspect backup {}", path.display())),
        Ok(_) => bail!(
            "backup already exists; refusing to overwrite: {}",
            path.display()
        ),
    }
}

fn ensure_unchanged(edit: &PreparedMask) -> Result<()> {
    require_regular_file(&edit.path)?;
    let mut current = Vec::new();
    fs::File::open(&edit.path)?
        .take(edit.original.len() as u64 + 1)
        .read_to_end(&mut current)?;
    if current != edit.original {
        bail!(
            "refusing to mask {} because the file changed since scan",
            edit.display_path.display()
        );
    }
    Ok(())
}

fn write_temporary(
    path: &Path,
    content: &[u8],
    permissions: fs::Permissions,
) -> Result<tempfile::NamedTempFile> {
    let parent = path.parent().unwrap_or(Path::new("."));
    let mut tmp = tempfile::Builder::new()
        .prefix(".veil-mask.")
        .tempfile_in(parent)?;
    tmp.write_all(content)?;
    tmp.flush()?;
    fs::set_permissions(tmp.path(), permissions)?;
    tmp.as_file().sync_all()?;
    Ok(tmp)
}

fn masked_content(content: &str, findings: &[veil_core::Finding]) -> Result<String> {
    let mut by_line = BTreeMap::new();
    for finding in findings {
        if finding.masked_snippet.contains(['\r', '\n']) {
            bail!("masked snippet must not contain line endings");
        }
        by_line.entry(finding.line_number).or_insert(finding);
    }
    let mut output = String::with_capacity(content.len());
    for (index, segment) in content.split_inclusive('\n').enumerate() {
        let (line, ending) = if let Some(line) = segment.strip_suffix("\r\n") {
            (line, "\r\n")
        } else if let Some(line) = segment.strip_suffix('\n') {
            (line, "\n")
        } else {
            (segment, "")
        };
        if let Some(finding) = by_line.remove(&(index + 1)) {
            if line != finding.line_content {
                bail!("mask input changed since scan");
            }
            output.push_str(&finding.masked_snippet);
            output.push_str(ending);
        } else {
            output.push_str(segment);
        }
    }
    if !by_line.is_empty() {
        bail!("mask finding refers to a missing input line");
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn changed_snapshot_is_rejected_without_overwriting() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("input.txt");
        fs::write(&path, "changed\n").unwrap();
        let edit = PreparedMask {
            display_path: path.clone(),
            path: path.clone(),
            original: b"original\n".to_vec(),
            replacement: "masked\n".into(),
            findings: 1,
            permissions: fs::metadata(&path).unwrap().permissions(),
            backup: None,
        };
        assert!(ensure_unchanged(&edit).is_err());
        assert_eq!(fs::read_to_string(path).unwrap(), "changed\n");
    }
}
