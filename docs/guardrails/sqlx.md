# CI Guardrail: SQLx Prepare

The **SQLx Guardrail** ensures that all **compile-time checked SQL queries** in the codebase are valid and that their offline cache (typically `sqlx-data.json`) is consistent with the code.

## What Runs in CI?

In the `stable` job, we first install `sqlx-cli` (pinned version), then run:

```bash
SQLX_OFFLINE=true cargo sqlx prepare --check --workspace 2>&1 | tee .local/ci/sqlx_prepare_check.txt
```

### Why `SQLX_OFFLINE=true`?

It forces `sqlx` to check queries against the offline cache file(s) committed in the repository (e.g., `sqlx-data.json`) instead of a live database. This ensures:

1.  **Cache Consistency**: Detects if you modified SQL queries in Rust code but forgot to update the offline cache.
2.  **Offline Buildability**: Ensures the code can verify query types in environments without a running database.

## Recovery (Shortest Path)

If this check fails (typically with "query data is out of date" or similar), follow these steps locally:

1.  **Ensure DB is Up**:
    ```bash
    cargo sqlx migrate run
    ```

2.  **Update Offline Data**:
    ```bash
    # Runs `cargo sqlx prepare` (without --check) to rewrite the cache file(s)
    cargo sqlx prepare --workspace -- --all-targets
    ```
    *Note: The generated file might be `sqlx-data.json` or split across crates depending on configuration.*

3.  **Commit**:
    Commit the updated cache file(s).
