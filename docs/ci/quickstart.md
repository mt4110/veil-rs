# CI/CD Integration Quickstart

Automate secret detection in your pipeline to catch secrets before they are merged.

## Minimal Template (GitHub Actions)

Copy to `.github/workflows/veil-scan.yml`:

```yaml
name: Veil Scan

on:
  push:
    branches: [ "main" ]
  pull_request:

jobs:
  veil-scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo install --git https://github.com/mt4110/veil-rs.git --tag v0.7.0
      - run: veil scan . --format json --fail-on-severity High > veil-report.json
```

## Failure Flags

Control when the CI job fails:

*   `--fail-on-severity <LEVEL>`: Fail if any finding matches or exceeds level (Low | Medium | High | Critical).
*   `--fail-on-score <INT>`: Fail if any finding score >= threshold (0-100).
*   `--fail-on-findings <N>`: Fail if the total number of findings >= N. Useful for monitoring "explosive" increases.

## Recommended Set

For most projects, aim to block High severity secrets while monitoring overall trends:

```bash
veil scan . --format json --fail-on-severity High --fail-on-score 90
```
