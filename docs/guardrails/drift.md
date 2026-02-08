# Drift Check Specification

## Scope
The Drift Check ensures the repository state matches the guardrails policy.
If the state drifts, `nix run .#prverify` fails.

### 1. CI Drift
Ensures `.github/workflows/ci.yml` is configured correctly.
- **Must**: Use `ops/ci/install_sqlx_cli.sh` for installation.
- **Must**: Generate and upload `.local/ci/` logs (`sqlx_cli_install.log`, `sqlx_prepare_check.txt`).
- **Must**: Create `.local/ci/.keep`.

### 2. Docs Drift
Ensures documentation contains necessary keywords for maintenance.
- **Must**: `docs/guardrails/sqlx.md` or `docs/ci/prverify.md` must mention `SQLX_OFFLINE`, `sqlx_cli_install.log`, and `ops/ci/`.

### 3. Registry Drift
Ensures `ops/exceptions.toml` exists and is valid.
- **Must**: File must exist and be valid TOML.
- **Must**: Compliance with schema (required fields, scope grammar, date formats).
- *Note: PR42 enforces schema only; expiry enforcement is PR43+.*

### 4. SOT Drift
Ensures a valid Source of Truth (SOT) exists and contains evidence for the guardrails.
- **Must**: At least one SOT candidate file matching `docs/pr/PR-<digits>-*.md` must exist.
- **Must**: The **selected** SOT must contain evidence keywords (`sqlx_cli_install.log`, `SQLX_OFFLINE`).
- **Rule**: If SOT is missing, ambiguous, or evidence is incomplete, **FAIL**. SOT is the evidence contract of the release.

### 5. SOT Selection Rules
To avoid guessing, `drift-check` selects the SOT deterministically:
- **Filename**: Must match `docs/pr/PR-<digits>-*.md`.
- **Priority**:
  1. **Exact Match (optional)**: If a PR number is provided as context, that PR is selected.
  2. **Max PR (default)**: Otherwise, the candidate with the **highest** PR number is selected.
  - Provide PR context via `--wanted-pr <N>` (0 = auto).
    - Example (Nix): `nix run .#prverify -- --wanted-pr 35`
- **Ambiguity**: If multiple candidates exist for the *same* PR number (duplicate), **FAIL** (`sot_ambiguous`).
- **Missing**: If no candidates found, **FAIL** (`sot_missing`).

## Runbook (Quick Fix)

If `drift-check` fails locally or in CI, look at the **1-scroll error block** at the end.

| Category | Typical Cause | Fix Command (Example) |
| :--- | :--- | :--- |
| **CI** | Workflow or `ops/` drift | `git checkout main .github/workflows/ci.yml` |
| **Docs** | Missing policy terms | `grep -r "SQLX_OFFLINE" docs/` |
| **Registry** | Missing or invalid `ops/exceptions.toml` | `mkdir -p ops && touch ops/exceptions.toml` |
| **SOT** | Missing/Duplicate/No Evidence | `ls docs/pr/` or edit the latest SOT |

### Recovery Steps:
1. **Identify**: Check the `Reason:`, `Fix:`, and `Next:` fields in the CLI output.
   ```text
   Reason: ...
   Fix:    ...
   Next:   nix run .#prverify
   ```
2. **Execute**: Run the `Fix` command.
3. **Verify**: Run the `Next` command.

## Handling Exceptions (.driftignore)

For temporary workarounds or legacy acceptance, use structured exceptions in `.driftignore`.

### Format
`substring # <reason> | until_YYYYMMDD`

- **substring**: The unique part of the error message to ignore.
- **reason**: Why this exception is needed (Audit required).
- **until**: Expiration date (Maintenance required).

### Example
```text
# Temporary ignore until v0.23.0 cleanup
sot_missing # Pending PR-39 creation | until_20260301
```

> [!IMPORTANT]
> Expired exceptions will trigger a **Warning** and should be removed as technical debt.

## Non-Goals
- Expanding drift check to arbitrary files without a clear policy.
- Parsing every file in the repo (performance).
