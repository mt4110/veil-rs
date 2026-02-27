# veil-rs 🛡️

**veil-rs** is a high-performance, local-first secret scanning tool designed for developers and enterprise security teams. It detects sensitive information (API keys, PII, credentials) in your codebase with incredible speed and flexibility.

## Features
- **🚀 Blazing Fast**: Built with Rust, utilizing parallel processing and regex optimization.
- **🇯🇵 Japan PII Ready**: Specialized detection for MyNumber, Driver's Licenses, and local PII formats.
- **👮 Commercial Grade**: Custom rules via TOML, inline ignores, and commercial-ready reporting.
- **📊 Reporting**: Output to JSON or beautiful HTML dashboards (`--format html`).
- **🛡️ Robust Controls**: `--fail-on-score` for CI pass/fail logic, enabling flexible `WARN` level operations.
- **⚡ Staged Scan**: `--staged` option scans only files staged for commit. Perfect for `pre-commit`.
- **📦 Binary Safe**: Automatically skips binary files and files >1MB to prevent CI bottlenecks.
- **🔧 Configurable & Layered**: Supports `veil.toml` and organization-wide policy layers (`VEIL_ORG_CONFIG`).
- **🩹 Baseline Support**: Easily suppress known violations using `.veil-baseline.json`.

### 🛡️ Data Privacy & Local-First Guarantee
Veil is designed for B2B environments and strictly adheres to a "Local-First" architecture:
- **Zero Telemetry**: Veil never sends your code, findings, or analytics to external servers.
- **Local Triage**: The Veil Pro Dashboard runs entirely on `localhost` (127.0.0.1) and enforces strict Content-Security-Policy rules preventing external assets (CDNs) or token leakage.
- **Isolated Operations**: All scanning, baseline generation, and reporting stay completely within your local machine or CI runner environment.

## Canonical Rules: RulePack (Source of Truth)

Veil’s rules are canonically defined as **RulePacks** (a directory with `00_manifest.toml` + one or more TOMLs containing `[[rules]]`).

For log scrubbing, generate a repo-local Log RulePack:

```bash
# Recommended: install from a pinned release tag
cargo install --locked --git https://github.com/mt4110/veil-rs.git --tag v1.0.0 veil-cli

# Dev (Nix): build from this repo
nix develop
cargo install --path crates/veil-cli
```
> **Note (Windows users):** You don't need Nix, but **Rust (Cargo) is required**. Just run `cargo install --path crates/veil-cli`.

### 2. Go to YOUR project

Leave the Veil repository and go to the project you want to scan.

```bash
# From source
git clone https://github.com/mt4110/veil-rs.git
cd veil-rs
cargo install --path crates/veil-cli --bin veil
```

> [!IMPORTANT]
> **For Developers: Use Nix Environment**
> This project is designed to be developed inside `nix develop`.
> Using a system-level Rust toolchain (e.g., v1.82.0 or older) may cause build failures due to new dependency requirements (like Rust 2024 Edition).
> Always use `nix develop` to ensure you are using the correct toolchain version.

## Usage

### Basic Scan
```bash
veil scan .
```

### Veil Pro Dashboard Quickstart
The **Veil Pro Dashboard** acts as your local "command center" for daily triage, noise management, and auditing, featuring B2B-grade security measures like local-first isolation and Zero Telemetry.

1. **Start the Dashboard**:
```bash
cargo run -p veil-pro
```
2. **Access securely**: The server exclusively binds to `127.0.0.1`. Open the URL printed to `stderr` (e.g. `http://127.0.0.1:3000/#token=xxxxxxxx`). The `#token` fragment ensures credentials never leak in server logs or browser history.
3. **DO NOT expose externally**: Do not proxy this server to the internet or 0.0.0.0.
4. **Audit-Ready Evidence**: Use the "Export Evidence Pack" button on the UI to securely generate a ZIP file of `report.html`, `report.json`, configurations, and `run_meta` for compliance auditing.

### 🕵️ Third-Party Verification (Golden Path)
Veil enables zero-trust compliance via the `verify` command. Auditors can verify evidence packs locally without network access or relying on blind trust.

1. **Generate Evidence**: Export the `evidence.zip` via the Dashboard.
2. **Anchor the Trust**: Extract `run_meta.json` and compute its **raw byte SHA256**. Record this hash in your ticketing system or audit log:
   ```bash
   unzip -p evidence.zip run_meta.json | shasum -a 256
   # e.g. e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
   ```
3. **Verify Integrity**: The auditor downloads the ZIP and runs verification, explicitly anchoring trust to the recorded hash.
   ```bash
   veil verify evidence.zip \
     --expect-run-meta-sha256 e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 \
     --require-complete
   ```
* `Exit 0`: The pack is structurally intact, untampered, complete, and contains zero token leakage.
* `Exit 1`: Valid pack, but failed strict operational policy (e.g., incomplete scan).
* `Exit 2`: Broken, dangerous (ZipSlip/ZipBomb), corrupted hashes, or token leakages (`#token=`) detected.

## 🔏 Evidence Pack Signing (Audit)
For audit submission, use an external anchor + signature:

1) Record `run_meta.json` SHA256 (external anchor)
   ```bash
   unzip -p evidence.zip run_meta.json | shasum -a 256
   ```
2) Sign `evidence.zip` (minisign or SSH)
3) Third-party verifies signature + runs:
   ```bash
   veil verify evidence.zip --expect-run-meta-sha256 <hash>
   ```

See `docs/design/160_evidence_signing_playbook.md` for the full playbook.

### JSON Output
```bash
veil scan . --format json
```

### HTML Dashboard (Enterprise Report)
```bash
veil scan . --format html > report.html
open report.html
```

## Commercial Usage Guide

### 1. Custom Rules (Pure TOML)
Define your own rules directly in `veil.toml` without modifying source code.

```toml
[rules.internal_project_id]
enabled = true
description = "Internal Project ID (PROJ-XXXX)"
pattern = "PROJ-\\d{4}"
severity = "high"
score = 80
category = "internal"
tags = ["proprietary"]
```

### 2. Inline Ignore (False Positive Handling)
Suppress findings directly in code using comments.

```rust
let fake_key = "AKIA1234567890"; // veil:ignore
let test_token = "ghp_xxxxxxxx"; // veil:ignore=rule_id
```

*   `// veil:ignore`: Ignore all findings on this line.
*   `// veil:ignore=rule_id`: Ignore only the specified rule ID.

## Testing

veil-rs includes tests for secret detection rules (Slack, AWS, GitHub PATs, etc.).

To avoid GitHub Push Protection blocking pushes, we **never** hard-code real-looking secrets
as string literals. Instead, tests generate fake tokens at runtime via helper functions.

See [docs/TESTING_SECRETS.md](docs/TESTING_SECRETS.md) for the full “Safety Contract”
and guidelines on adding new secret tests.

### 3. Policy Layering (Organization Rules)
Manage organization-wide blocklists or allowance settings centrally.
Set the `VEIL_ORG_CONFIG` environment variable to point to a shared config file. It merges with project-level `veil.toml` (project overrides org).

```bash
export VEIL_ORG_CONFIG=/etc/veil/org_policy.toml
# Configuring "fail_on_score = 50" in org_policy.toml enforcing strict checks across all projects.
```
> **Note:** The legacy `VEIL_ORG_RULES` environment variable is **Deprecated** and will be removed in future versions. Please migrate to `VEIL_ORG_CONFIG`.

### 3. CI/CD Integration
Drop-in templates are available in `examples/ci/`.

**GitHub Actions:**
```yaml
- name: Run Veil Scan
  run: |
    veil scan . --format html > report.html
    # Fail CI if score >= 80
    veil scan . --fail-on-score 80
    # Or scan only staged files (for PRs)
    # veil scan --staged
```


### 4. Git Hook (pre-commit)
You can automatically scan before committing using `pre-commit`.
Add the following to your `.pre-commit-config.yaml`:

```yaml
repos:
  - repo: local
    hooks:
      - id: veil-scan
        name: veil-scan
        entry: veil scan
        language: system
        types: [text]
        exclude: '\.git/|\.png$|\.jpg$'
```

## License
Dual licensed under Apache 2.0 or MIT.

> **Note**: While currently provided as OSS under MIT/Apache-2.0, future versions introducing advanced enterprise-grade features may adopt different licensing models or paid add-ons (the v0.x series will remain OSS). We are exploring optimal models for sustainable OSS development.
