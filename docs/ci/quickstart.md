# CI/CD Integration Quickstart

Automate secret detection in your pipeline to catch secrets before they are merged.

## Minimal Template (GitHub Actions)

Copy to `.github/workflows/veil-scan.yml`:

Use the latest **stable** release tag (no `-rc`), e.g. `v0.17.0`.

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
      - run: cargo install --locked --git https://github.com/mt4110/veil-rs.git --tag vX.Y.Z veil-cli
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

## Storing HTML Reports (GitHub Actions)

You can save the HTML report as a build artifact to view scan results in detail.

```yaml
# .github/workflows/veil.yml
name: Veil Scan

on:
  push:
    branches: [ main ]
  pull_request:

jobs:
  scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install veil
        run: |
          cargo install --locked --git https://github.com/mt4110/veil-rs.git --tag vX.Y.Z veil-cli

      - name: Run veil scan (HTML)
        run: |
          veil scan . --format html > veil-report.html

      - name: Upload veil report
        uses: actions/upload-artifact@v4
        with:
          name: veil-report
          path: veil-report.html
```
