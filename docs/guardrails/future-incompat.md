# CI Guardrail: Future Incompatibilities

The **Future Incompatibility** guardrail ensures that the codebase remains forward-compatible with upcoming Rust releases.

## What Runs in CI?

In the `stable` job of our CI pipeline (`.github/workflows/ci.yml`), the following check runs before tests:

```bash
cargo check --workspace --all-targets --future-incompat-report 2>&1 | tee .local/ci/check_future_incompat.log
```

If the log does **not** contain the message "note: 0 dependencies had future-incompatible warnings", CI will generate a report:

```bash
cargo report future-incompatibilities | tee .local/ci/future_incompat.txt
```

## The "Zero Warnings" Rule

- **0 warnings**: The check passes silently.
- **>0 warnings**: The check still passes (for now), but generates a report in the artifacts.

**Policy**: We aim for 0 warnings. If new warnings appear, they must be addressed promptly to prevent breakage in future Rust versions.

## Troubleshooting

If this step highlights issues:

1.  **Check Artifacts**: Download the `guardrail-logs` artifact from the GitHub Actions run.
2.  **Read Logs**:
    - `.local/ci/check_future_incompat.log`: Raw output of the check.
    - `.local/ci/future_incompat.txt`: Detailed report of the incompatibilities.
3.  **Fix Locally**:
    Run `cargo report future-incompatibilities --id <id>` (as guided by the output) to understand the required changes, such as upgrading dependencies.
