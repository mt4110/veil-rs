use assert_cmd::Command;
use predicates::prelude::*;
use std::fs::File;
use std::io::Write;
use tempfile::TempDir;

#[test]
fn test_policy_layering_fail_score() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();

    // Create a secret file (Score ~80 for AWS key)
    let secret_file = repo_path.join("secret.txt");
    let mut f = File::create(&secret_file)?;
    f.write_all(b"aws_key = \"AKIA1234567890AVCDEF\"")?;

    // 1. Create Org Config (Fail on 50)
    let org_config_path = temp_dir.path().join("org_rules.toml");
    let mut f_org = File::create(&org_config_path)?;
    writeln!(f_org, "[core]\nfail_on_score = 50")?;

    // 2. Run with VEIL_ORG_RULES, no project config
    // Should FAIL (Score 80 > 50)
    #[allow(deprecated)]
    let mut cmd = Command::cargo_bin("veil-cli")?;
    cmd.current_dir(repo_path)
        .env("VEIL_ORG_RULES", org_config_path.to_str().unwrap())
        .arg("scan")
        .assert()
        .failure();

    // 3. Create Org Config (Fail on 150)
    let org_config_safe = temp_dir.path().join("org_safe.toml");
    let mut f_org_safe = File::create(&org_config_safe)?;
    writeln!(f_org_safe, "[core]\nfail_on_score = 150")?;

    // 4. Run with High Threshold
    // Should PASS (Score 80 < 100)
    #[allow(deprecated)]
    let mut cmd2 = Command::cargo_bin("veil-cli")?;
    cmd2.current_dir(repo_path)
        .env("VEIL_ORG_RULES", org_config_safe.to_str().unwrap())
        .arg("scan")
        .assert()
        .success();

    Ok(())
}

#[test]
fn test_policy_layering_override() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();

    // Secret file
    let secret = "AKIA1234567890AVCDEF";
    let secret_file = repo_path.join("secret.txt");
    let mut f = File::create(&secret_file)?;
    writeln!(f, "{}", secret)?;

    // Org Config: Fail on 50 (Strict)
    let org_config_path = temp_dir.path().join("org.toml");
    let mut f_org = File::create(&org_config_path)?;
    writeln!(f_org, "[core]\nfail_on_score = 50")?;

    // Project Config: Fail on 150 (Loose, Override)
    let project_config_path = repo_path.join("veil.toml");
    let mut f_proj = File::create(&project_config_path)?;
    writeln!(f_proj, "[core]\nfail_on_score = 150")?;

    // Run
    // Should PASS because Project config (100) overrides Org (50), and 80 < 100.
    let mut cmd = Command::cargo_bin("veil-cli")?;
    cmd.current_dir(repo_path)
        .env("VEIL_ORG_RULES", org_config_path.to_str().unwrap())
        .arg("scan")
        .assert()
        .success();

    Ok(())
}

#[test]
fn test_policy_layering_ignore_extend() -> Result<(), Box<dyn std::error::Error>> {
    let temp_dir = TempDir::new()?;
    let repo_path = temp_dir.path();

    // File A: ignored by Org
    let file_a = repo_path.join("org_ignore.txt");
    let mut fa = File::create(&file_a)?;
    writeln!(fa, "AKIA1234567890AVCDEF")?;

    // File B: ignored by Project
    let file_b = repo_path.join("proj_ignore.txt");
    let mut fb = File::create(&file_b)?;
    writeln!(fb, "AKIA1234567890AVCDEF")?;

    // Org Config
    let org_path = temp_dir.path().join("org.toml");
    let mut fo = File::create(&org_path)?;
    writeln!(fo, "[core]\nignore = [\"org_ignore.txt\"]")?;

    // Project Config
    let proj_path = repo_path.join("veil.toml");
    let mut fp = File::create(&proj_path)?;
    writeln!(fp, "[core]\nignore = [\"proj_ignore.txt\"]")?;

    // Run
    // Should ignore BOTH files and succeed (no findings).
    // If it didn't merge properly, one would be detected.
    let mut cmd = Command::cargo_bin("veil-cli")?;
    cmd.current_dir(repo_path)
        .env("VEIL_ORG_RULES", org_path.to_str().unwrap())
        .arg("scan")
        .assert()
        .success();

    Ok(())
}
