# veil-rs 🛡️

**veil-rs** is a high-performance, local-first secret scanning tool designed for developers and enterprise security teams. It detects sensitive information (API keys, PII, credentials) in your codebase with incredible speed and flexibility.

## Features
- **🚀 Blazing Fast**: Built with Rust, utilizing parallel processing and regex optimization.
- **🇯🇵 Japan PII Ready**: Specialized detection for MyNumber, Driver's Licenses, and local PII formats.
- **👮 Commercial Grade**: Custom rules via TOML, inline ignores, and commercial-ready reporting.
- **📊 Reporting**: Output to JSON or beautiful HTML dashboards (`--format html`).
- **🔧 Configurable**: Fully customizable via `veil.toml`.

## Installation

```bash
# From source
git clone https://github.com/takem1-max64/veil-rs.git
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
let test_token = "ghp_xxxxxxxx"; // veil:ignore=github_personal_access_token
```

### 3. CI/CD Integration
Drop-in templates are available in `examples/ci/`.

**GitHub Actions:**
```yaml
- name: Run Veil Scan
  run: |
    veil scan . --format html > report.html
    veil scan . --fail-score 80
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
