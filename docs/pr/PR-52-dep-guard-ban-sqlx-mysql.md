# PR52 Dependency Guard (Ban sqlx-mysql)

- Scope:
  - Implement `dep-trace` tool to trace dependencies in `Cargo.lock`.
  - Add "Dependency Guard" step to `prverify` to ban `sqlx-mysql`.
  - Refactor `veil-server` to remove `sqlx-mysql` (replace `sqlx` facade with `sqlx-postgres`/`sqlx-core`).
- Non-goals:
  - General dependency auditing (focused on sqlx-mysql for now).
- Verification:
  - `nix run .#prverify` (PASS)
  - `dep-trace` check (PASS)

Latest prverify report: docs/evidence/prverify/prverify_20260211T021046Z_fe0205b.md
