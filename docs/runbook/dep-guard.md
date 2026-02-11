# Dependency Guard

## Why is this guard here?

We strictly ban `sqlx-mysql` from our dependency tree for the following reasons:
1.  **Security**: It pulls `rsa` < 0.9.x, which has known vulnerabilities (RUSTSEC-2023-0071).
2.  **Cleanliness**: Veil is a PostgreSQL-only application. Linking unused database drivers bloats the binary and attack surface.
3.  **Stability**: `sqlx` features are additive. If one crate enables `mysql`, it infects the entire workspace.

## How to Read the Output

When `prverify` fails with `Dependency Guard found issues`:

```text
LOCK TRACE:
veil-server
  -> sqlx
    -> sqlx-mysql
```

- **Top level**: The workspace member (e.g., `veil-server`) that is responsible.
- **Middle**: The chain of dependencies.
- **Bottom**: The banned crate (`sqlx-mysql`).

## How to Fix

1.  **Identify the Culprit**: Look at the "Top level" crate in the trace.
2.  **Check `Cargo.toml`**: Open that crate's `Cargo.toml`.
3.  **Inspect `sqlx` Features**:
    - Ensure `default-features = false`.
    - Ensure **neither** `mysql`, `all-databases`, nor `any` are enabled.
4.  **Update Lockfile**:
    - Run `cargo update -p <dependency>` to refresh `Cargo.lock`.
    - Verification: `rg 'name = "sqlx-mysql"' Cargo.lock` should return nothing.
5.  **Regenerate (Last Resort)**:
    - If minimal update fails, `git restore Cargo.lock && cargo generate-lockfile`. (Caution: This updates *everything*).
