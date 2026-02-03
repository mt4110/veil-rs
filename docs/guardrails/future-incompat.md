# CI Guardrail: Future Incompatibilities

This guardrail keeps the repo forward-compatible with upcoming Rust releases by watching for **future-incompatible warnings** (warnings that may become hard errors in a future Rust).

## What runs in CI?

In the `stable` job (`.github/workflows/ci.yml`), CI runs:

```bash
# 1. Check for warnings and save raw output (always created)
cargo check --workspace --all-targets --future-incompat-report 2>&1 | tee .local/ci/check_future_incompat.log

# 2. Try to generate a report
# If no report exists, `cargo report` exits non-zero (101) with "error: no reports are currently available".
# This is considered NORMAL (no warnings observed).
if cargo report future-incompatibilities > .local/ci/future_incompat.txt 2>&1; then
  echo "✅ future-incompat report saved"
else
  # If 101/no report, remove the empty file if created
  rm -f .local/ci/future_incompat.txt
  echo "✅ no future-incompat report available (NORMAL)"
fi
```

### Artifacts (guardrail-logs)

*   **.local/ci/check_future_incompat.log**: **Always present**. Raw output of the check.
*   **.local/ci/future_incompat.txt**: **Only present if warnings exist**. Detailed report.

## The "Zero Warnings" Rule

*   **No Report (Normal)**: The check passes.
*   **Report Exists**: The check still passes (for now), but we should fix it.

**Policy**: We aim for 0 warnings. If `.local/ci/future_incompat.txt` appears in artifacts, investigate immediately.

## Recovery (Shortest Path)

To check status locally:

```bash
# Check if report exists
test -f .local/ci/future_incompat.txt && cat .local/ci/future_incompat.txt || echo "OK: no report (normal)"
```

To fix issues, run the report command locally to get IDs, then fix the code:

```bash
cargo check --future-incompat-report
cargo report future-incompatibilities --id <ID>
```