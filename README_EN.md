# veil-rs 🛡️

**veil-rs** is a high-performance, local-first secret scanning tool designed for developers and enterprise security teams. It detects sensitive information (API keys, PII, credentials) in your codebase with incredible speed and flexibility.

## Features
- **🚀 Blazing Fast**: Built with Rust, utilizing parallel processing and regex optimization.
- **🇯🇵 Japan PII Ready**: Specialized detection for MyNumber, Driver's Licenses, and local PII formats.
- **👮 Commercial Grade**: Custom rules via TOML, inline ignores, and commercial-ready reporting.
- **📊 Reporting**: Output to JSON or beautiful HTML dashboards (`--format html`).
- **🛡️ Robust Controls**: `--fail-score` for CI pass/fail logic, enabling flexible `WARN` level operations.
- **⚡ Staged Scan**: `--staged` option scans only files staged for commit. Perfect for `pre-commit`.
- **📦 Binary Safe**: Automatically skips binary files and files >1MB to prevent CI bottlenecks.
- **🔧 Configurable & Layered**: Supports `veil.toml` and organization-wide policy layers (`VEIL_ORG_RULES`).

## Installation

```bash
# From source
git clone https://github.com/mt4110/veil-rs.git
cd veil-rs
cargo install --path crates/veil-cli --bin veil
```

## Usage

### Basic Scan
```bash
veil scan .
```

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
let test_token = "ghp_xxxxxxxx"; *   `// veil:ignore`: Ignore all findings on this line.
*   `// veil:ignore=rule_id`: Ignore only the specified rule ID.

## Testing

veil-rs includes tests for secret detection rules (Slack, AWS, GitHub PATs, etc.).

To avoid GitHub Push Protection blocking pushes, we **never** hard-code real-looking secrets
as string literals. Instead, tests generate fake tokens at runtime via helper functions.

See [docs/TESTING_SECRETS.md](docs/TESTING_SECRETS.md) for the full “Safety Contract”
and guidelines on adding new secret tests.
### 3. Policy Layering (Organization Rules)
Manage organization-wide blocklists or allowance settings centrally.
Set the `VEIL_ORG_RULES` environment variable to point to a shared config file. It merges with project-level `veil.toml` (project overrides org).

```bash
export VEIL_ORG_RULES=/etc/veil/org_policy.toml
# Configuring "fail_on_score = 50" in org_policy.toml enforcing strict checks across all projects.
```

### 3. CI/CD Integration
Drop-in templates are available in `examples/ci/`.

**GitHub Actions:**
```yaml
- name: Run Veil Scan
  run: |
    veil scan . --format html > report.html
    veil scan . --format html > report.html
    # Fail CI if score >= 80
    veil scan . --fail-score 80
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
