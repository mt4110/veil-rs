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

### 3. SOT Drift
Ensures a valid Source of Truth (SOT) exists for the current release.
- **Must**: A file matching `docs/pr/*v0.22.0*robust-sqlx*.md` must exist.
- **Must**: The SOT must contain evidence keywords (`sqlx_cli_install.log`, `SQLX_OFFLINE`).
- **Rule**: If SOT is missing or mismatched, **FAIL**. SOT is the contract of the release.

### 4. SOT Selection Rules
To avoid ambiguity, `drift-check` selects the SOT deterministically:
- **Filename**: Must match `docs/pr/PR-<digits>-*.md`.
- **Priority**:
  1. **Exact Match**: If the PR number is specified (known context), that file is selected.
  2. **Max PR**: If context is unknown, the file with the **highest** PR number is selected.
- **Ambiguity**: If multiple candidates exist for the *same* PR number (duplicate), **FAIL** (`sot_ambiguous`).
- **Missing**: If no candidates found, **FAIL** (`sot_missing`).

## Runbook
If `drift-check` fails:

1. **Read the Log**: Look for `drift=TYPE reason=...`.
   - `drift=CI`: CI config was changed illegally. Revert or update `ops/ci` scripts.
   - `drift=Docs`: Docs are missing key terms. Restore them.
   - `drift=SOT`: SOT is missing or evidence is incomplete.
2. **Fix**:
   - If SOT missing/drifted: Restore the file or update it to match reality (sync commit SHA).
3. **Verify**:
   - Run `nix run .#prverify`.
   - Ensure it passes (Green).

## Non-Goals
- Expanding drift check to arbitrary files without a clear policy.
- Parsing every file in the repo (performance).
