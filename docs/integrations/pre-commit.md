# pre-commit Integration

`veil-rs` can be used with [pre-commit](https://pre-commit.com) to automatically scan for secrets before they are committed.

## Prerequisites

Ensure `veil` is installed and available in your `PATH`.

```bash
cargo install --path crates/veil-cli --bin veil
# or
cargo install veil
```

## Configuration

Add the following to your `.pre-commit-config.yaml`:

```yaml
repos:
  - repo: https://github.com/mt4110/veil-rs
    rev: v0.6.1  # specific tag or main
    hooks:
      - id: veil-scan
        # Optional: Fail on any finding, even low severity
        # args: ["--fail-on-findings"]
```

## Hooks

*   **`veil-scan`**: Scans the files staged for commit. Fast and efficient for daily dev.
*   **`veil-scan-all`**: Scans the entire repository. Useful for CI or occasional full checks.
