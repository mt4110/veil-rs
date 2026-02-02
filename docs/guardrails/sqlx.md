# CI Guardrail: SQLx Prepare

The **SQLx Guardrail** ensures that all SQL queries in the codebase are valid and that the offline query cache (`sqlx-data.json`) is up-to-date.

## What Runs in CI?

In the `stable` job, the following command runs:

```bash
SQLX_OFFLINE=true cargo sqlx prepare --check --workspace
```

## Intent

- **Verify Queries**: Ensures compile-time checked queries are valid against the schema.
- **Check Cache**: Ensures `sqlx-data.json` matches the actual queries in the code. This is crucial because we build in offline mode in some environments (and CI often needs it).

## Recovery (Shortest Path)

If this check fails in CI (usually due to "query data is out of date"), follow these steps:

1.  **Ensure Database is Up**:
    ```bash
    # Ensure your local DB is running and migrated
    cargo sqlx migrate run
    ```

2.  **Update Offline Data**:
    ```bash
    cargo sqlx prepare --workspace -- --all-targets
    ```

3.  **Commit Changes**:
    Commit the updated `sqlx-data.json` file.
