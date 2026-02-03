# CI Guardrail: SQLx Prepare

The **SQLx Guardrail** ensures that all SQL queries in the codebase are valid and that the offline query cache (`sqlx-data.json`) is up-to-date.

## What Runs in CI?

In the `stable` job, we first install `sqlx-cli` (pinned version), then run:

```bash
SQLX_OFFLINE=true cargo sqlx prepare --check --workspace 2>&1 | tee .local/ci/sqlx_prepare_check.txt
```

### Why `SQLX_OFFLINE=true`?

It forces `sqlx` to check against `sqlx-data.json` instead of a live database. This ensures:
1.  The `sqlx-data.json` committed in the repo matches the code.
2.  The code will compile in environments without a running DB (like simple CI agents or during packaging).

## Recovery (Shortest Path)

If this check fails (typically "query data is out of date"), follow these steps locally:

1.  **Ensure DB is Up**:
    ```bash
    cargo sqlx migrate run
    ```

2.  **Update Offline Data**:
    ```bash
    # This updates sqlx-data.json
    cargo sqlx prepare --workspace -- --all-targets
    ```

3.  **Commit**:
    Commit the updated `sqlx-data.json`.
