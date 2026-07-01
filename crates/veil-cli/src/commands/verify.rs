use anyhow::Result;
use colored::Colorize;
use std::path::PathBuf;
use veil_core::verify::{verify_evidence_pack, VerifyOptions, VerifyStatus};

#[allow(clippy::too_many_arguments)]
pub fn verify(
    path: PathBuf,
    format: String,
    require_complete: bool,
    fail_on_findings: Option<usize>,
    expect_run_meta_sha256: Option<String>,
    max_zip_bytes: u64,
    max_entry_bytes: u64,
    max_total_bytes: u64,
    max_files: usize,
) -> Result<()> {
    if fail_on_findings == Some(0) {
        anyhow::bail!("--fail-on-findings must be >= 1");
    }

    let options = VerifyOptions {
        require_complete,
        fail_on_findings,
        expect_run_meta_sha256,
        max_zip_bytes,
        max_entry_bytes,
        max_total_bytes,
        max_files,
    };

    let result = verify_evidence_pack(&path, &options);

    match result {
        Ok(verify_result) => {
            let json_mode = format.to_lowercase() == "json";

            if json_mode {
                let out = serde_json::json!({
                    "status": "VALID",
                    "code": match verify_result.status {
                        VerifyStatus::Ok => 0,
                        VerifyStatus::PolicyViolation => 1,
                        VerifyStatus::Error => 2,
                    },
                    "message": verify_result.message,
                    "findings_count": verify_result.findings_count,
                    "is_complete": verify_result.is_complete,
                });
                println!("{}", serde_json::to_string_pretty(&out)?);
            } else if verify_result.status == VerifyStatus::Ok {
                println!(
                    "{} {}",
                    "✅ Evidence Pack Validation:".green().bold(),
                    "PASSED".green()
                );
                println!("{} {}", "ℹ Message:".blue(), verify_result.message);
                println!("   Findings: {}", verify_result.findings_count);
                println!("   Complete: {}", verify_result.is_complete);
            } else {
                println!(
                    "{} {}",
                    "❌ Evidence Pack Validation:".red().bold(),
                    "POLICY VIOLATION".red()
                );
                println!("{} {}", "ℹ Message:".blue(), verify_result.message);
                println!("   Findings: {}", verify_result.findings_count);
                println!("   Complete: {}", verify_result.is_complete);
            }

            match verify_result.status {
                VerifyStatus::Ok => std::process::exit(0),
                VerifyStatus::PolicyViolation => std::process::exit(1),
                VerifyStatus::Error => std::process::exit(2), // Should not reach here via Ok
            }
        }
        Err(e) => {
            let json_mode = format.to_lowercase() == "json";

            if json_mode {
                let out = serde_json::json!({
                    "status": "INVALID",
                    "code": 2,
                    "error": e.to_string(),
                });
                println!("{}", serde_json::to_string_pretty(&out)?);
            } else {
                println!(
                    "{} {}",
                    "🚨 Evidence Pack Verification FAILED:".red().bold(),
                    "CORRUPT OR DANGEROUS".red()
                );
                println!("   Reason: {}", e.to_string().yellow());
            }
            std::process::exit(2);
        }
    }
}
