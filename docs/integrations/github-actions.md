# GitHub Actions Integration

Automate secret scanning in your CI/CD pipeline using GitHub Actions.

## Example Workflow

Create `.github/workflows/veil-scan.yml` in your repository:

```yaml
name: Veil Security Scan

on:
  push:
    branches: [ "main", "master" ]
  pull_request:
    branches: [ "main", "master" ]

env:
  CARGO_TERM_COLOR: always

jobs:
  scan:
    name: Secret Scan
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      # Install Rust toolchain (often pre-installed, but good for stability)
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable

      # Install Veil (Build from source)
      # Note: In the future, we may provide a pre-built Action or binary release.
      - name: Install Veil
        run: cargo install --git https://github.com/mt4110/veil-rs crates/veil-cli --bin veil

      # Run Scan
      - name: Run Veil Scan
        run: |
          # Fail if any secrets found (--fail-on-findings)
          # Limit output to 2000 findings to avoid log explosion
          # Securely output JSON to a file (avoid printing secrets to stdout logs)
          veil scan . --format json --limit 2000 --fail-on-findings > veil-report.json
```

## Advanced Usage

### Custom Config

If you have a `veil.toml` in your repo root, it will be automatically picked up.

### Ignoring False Positives

Use the header comments or `veil:ignore` comments in code to suppress known issues.

```rust
let test_token = "fake_token"; // veil:ignore
```
