# Pre-commit Hook Integration

Veil provides two ways to integrate with git hooks:

1.  **Native Hook**: Using `veil pre-commit init` (Recommended for standalone usage).
2.  **Framework**: Using the popular [pre-commit](https://pre-commit.com/) framework.

## 1. Native Hook (Recommended)

The easiest way to enforce security checks locally is to install the native git hook.

```bash
# Initialize the hook in your current repository
veil pre-commit init
```

This will create `.git/hooks/pre-commit` which runs `veil scan --staged` before every commit.
If a secret is detected, the commit will be blocked.

**To bypass the check (Emergency only):**
```bash
git commit --no-verify
```

## 2. Pre-commit Framework

If your team uses the `pre-commit` framework (Python), add the following to your `.pre-commit-config.yaml`:

```yaml
repos:
  - repo: https://github.com/mt4110/veil-rs
    rev: v0.7.0  # Use the latest version
    hooks:
      - id: veil-scan
```

### Prerequisite
You must have `veil` installed and available in your `PATH`.
Since `veil` is a compiled binary/Rust tool, we use `language: system` to invoke it.

## Troubleshooting

### "Commit blocked by Veil Security Check"
This means Veil detected potential secrets in your staged files.

1.  **Review the output**: Veil lists the file and line number of the secret.
2.  **Fix key**: Remove the secret or replace it with an environment variable.
3.  **False Positive?**:
    *   Add `// veil:ignore` to the line.
    *   Or add the file/pattern to `veil.toml` under `[ignore.paths]`.
