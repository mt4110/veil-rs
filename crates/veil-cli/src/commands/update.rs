use crate::cli::UpdateArgs;
use anyhow::{Context, Result};
use colored::*;
use ed25519_dalek::{Verifier, VerifyingKey};
use std::fs::File;
use std::io::Read;
use std::path::{Component, PathBuf};
use zip::ZipArchive;

pub fn update(args: &UpdateArgs) -> Result<()> {
    eprintln!("{} Starting Veil RulePack Update...", "🔄".cyan());

    if args.url.is_some() {
        anyhow::bail!(
            "Remote downloads are disabled. Use --local-pack <archive.zip> --public-key <key.hex>."
        );
    }
    let local_path = args.local_pack.as_ref().context(
        "A local signed RulePack is required. Use --local-pack <archive.zip> --public-key <key.hex>. Built-in rules need no update.",
    )?;
    let archive_bytes = std::fs::read(local_path).context("Failed to read local RulePack file")?;
    let sig_path = local_path.with_extension("sig");
    let signature_hex = std::fs::read_to_string(&sig_path)
        .context("Missing or unreadable .sig file alongside local RulePack")?;
    let signature_hex = signature_hex.trim();

    eprintln!("  {} Verifying Signature...", "🔐".blue());

    // Determine public key
    let pub_key_hex = if let Some(pk_path) = &args.public_key {
        std::fs::read_to_string(pk_path)
            .context("Failed to read specified public_key file")?
            .trim()
            .to_string()
    } else if let Ok(env_key) = std::env::var("VEIL_RULEPACK_PUBKEY") {
        env_key
    } else {
        anyhow::bail!(
            "❌ No trust anchor configured.\n\
             Provide one of:\n\
             --public-key <path>    Path to a hex-encoded Ed25519 public key file\n\
             VEIL_RULEPACK_PUBKEY   Environment variable with hex-encoded public key\n\
             Refusing to install unsigned RulePack without explicit key pinning."
        );
    };

    let pub_key_bytes =
        hex::decode(&pub_key_hex).context("Failed to decode public key from hex")?;
    if pub_key_bytes.len() != 32 {
        anyhow::bail!("Invalid public key length. Expected 32 bytes.");
    }

    let mut pk_array = [0u8; 32];
    pk_array.copy_from_slice(&pub_key_bytes);
    let verifying_key = VerifyingKey::from_bytes(&pk_array).context("Invalid public key format")?;

    let sig_bytes = hex::decode(signature_hex).context("Failed to decode signature from hex")?;
    let signature =
        ed25519_dalek::Signature::from_slice(&sig_bytes).context("Invalid signature length")?;

    verifying_key.verify(&archive_bytes, &signature).context(
        "❌ Signature verification FAILED! The RulePack may be tampered with or corrupted.",
    )?;

    eprintln!("  {} Signature verified successfully.", "✅".green());

    // 4. Extract the RulePack
    eprintln!("  {} Installing RulePack to .veil/rules/...", "📦".blue());
    let reader = std::io::Cursor::new(archive_bytes);
    let mut archive = ZipArchive::new(reader).context("Failed to parse downloaded file as ZIP")?;

    // Check the signed manifest before creating any installation state.
    {
        let mut manifest_file = archive
            .by_name("00_manifest.toml")
            .context("❌ Missing 00_manifest.toml in RulePack. Update aborted.")?;
        let mut content = String::new();
        manifest_file
            .read_to_string(&mut content)
            .context("❌ Failed to read 00_manifest.toml. Update aborted.")?;
        let manifest = veil_core::rules::pack::parse_manifest(&content)
            .context("❌ Failed to parse 00_manifest.toml. Update aborted.")?;
        let exp = manifest.pack.expires_at.context(
            "❌ RulePack has no 'expires_at' TTL. Refusing to install unprotected Pack.",
        )?;

        if chrono::Utc::now() > exp {
            anyhow::bail!(
                "❌ The downloaded RulePack has EXPIRED (expires_at: {}).\nUpdate aborted to prevent downgrade attacks.",
                exp.to_rfc3339()
            );
        }
        for name in &manifest.files {
            let path = std::path::Path::new(name);
            if path.as_os_str().is_empty()
                || path
                    .components()
                    .any(|c| !matches!(c, Component::Normal(_) | Component::CurDir))
            {
                anyhow::bail!("RulePack manifest contains an invalid rule path: {name}");
            }
        }
    }

    let target_dir = PathBuf::from(".veil/rules");
    if target_dir.symlink_metadata().is_ok() {
        anyhow::bail!(
            "Existing .veil/rules was preserved. Install into a fresh project or configure another core.rules_dir."
        );
    }
    std::fs::create_dir_all(".veil").context("Failed to create .veil directory")?;
    let candidate = tempfile::Builder::new()
        .prefix(".rules-install-")
        .tempdir_in(".veil")
        .context("Failed to create RulePack staging directory")?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => candidate.path().join(path),
            None => anyhow::bail!("RulePack contains an invalid ZIP entry path"),
        };
        if file
            .unix_mode()
            .is_some_and(|mode| mode & 0o170000 == 0o120000)
        {
            anyhow::bail!("RulePack contains a symbolic link entry");
        }

        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p)?;
                }
            }
            let mut outfile = File::options()
                .write(true)
                .create_new(true)
                .open(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }

    veil_core::rules::pack::load_rule_pack(candidate.path())
        .context("Extracted RulePack validation failed; existing rules were preserved")?;
    if target_dir.symlink_metadata().is_ok() {
        anyhow::bail!(
            "Existing .veil/rules was preserved; installation target appeared during update"
        );
    }
    std::fs::rename(candidate.path(), &target_dir)
        .context("Failed to publish validated RulePack; retry the update")?;

    eprintln!(
        "{} Verified RulePack installed in .veil/rules.",
        "✅".green()
    );
    eprintln!("Not activated automatically. Set rules_dir = \".veil/rules\" in the [core] section of your configuration, then run veil rules list to confirm.");
    Ok(())
}
