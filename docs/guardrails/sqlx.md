# CI Guardrail: SQLx Prepare

The **SQLx Guardrail** ensures that all **compile-time checked SQL queries** in the codebase are valid and that their offline cache (typically `sqlx-data.json`) is consistent with the code.

## What Runs in CI?

In the `stable` job (`.github/workflows/ci.yml`), we:
- **Version Pinning**: The script pins `sqlx-cli` (e.g., `0.8.6`) to avoid upstream breaking changes. You can override this by setting `SQLX_CLI_VERSION` environment variable.
- **Robustness**: It includes retry logic (`attempt 1/3`) and standardizes flags (`--locked`, `--no-default-features`).
- **Retries**: Installation is attempted up to 3 times with backoff to handle network instability.
- **Logging**: Installation logs are saved to `.local/ci/sqlx_cli_install.log` (upload-artifact `guardrail-logs`).

> [!NOTE]
> Exceptional Policy: While shell scripts (`.sh`) are generally forbidden in favor of Rust/Go/Nix, we allow scripts in `ops/ci/` (like `install_sqlx_cli.sh`) to maintain robust, OS-agnostic CI operations without over-engineering compilation steps.

Then we run the strict check:

```bash
SQLX_OFFLINE=true cargo sqlx prepare --check --workspace 2>&1 | tee .local/ci/sqlx_prepare_check.txt
```

### Why `SQLX_OFFLINE=true`?

It forces `sqlx` to check queries against the offline cache file(s) committed in the repository (e.g., `sqlx-data.json`) instead of a live database. This ensures:

1.  **Cache Consistency**: Detects if you modified SQL queries in Rust code but forgot to update the offline cache.
2.  **Offline Buildability**: Ensures the code can verify query types in environments without a running database.

## Recovery (Shortest Path)

If the **Install** step fails:
- Check `.local/ci/sqlx_cli_install.log` in artifacts.
- Rerun the workflow (transient network issues are common).

If the **Check** step fails (typically "query data is out of date"):
1.  **Ensure DB is Up**: `cargo sqlx migrate run`
2.  **Update Offline Data**: `cargo sqlx prepare --workspace -- --all-targets`
3.  **Commit**: Commit the updated cache file(s).
