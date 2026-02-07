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
Ensures a valid Source of Truth (SOT) exists and contains evidence for the guardrails.
- **Must**: At least one SOT candidate file matching `docs/pr/PR-<digits>-*.md` must exist.
- **Must**: The **selected** SOT must contain evidence keywords (`sqlx_cli_install.log`, `SQLX_OFFLINE`).
- **Rule**: If SOT is missing, ambiguous, or evidence is incomplete, **FAIL**. SOT is the evidence contract of the release.

### 4. SOT Selection Rules
To avoid guessing, `drift-check` selects the SOT deterministically:
- **Filename**: Must match `docs/pr/PR-<digits>-*.md`.
- **Priority**:
  1. **Exact Match (optional)**: If a PR number is provided as context, that PR is selected.
  2. **Max PR (default)**: Otherwise, the candidate with the **highest** PR number is selected.
- **Ambiguity**: If multiple candidates exist for the *same* PR number (duplicate), **FAIL** (`sot_ambiguous`).
- **Missing**: If no candidates found, **FAIL** (`sot_missing`).

## Runbook
If `drift-check` fails:

1. **Read the Log**: Look for `ERROR: drift check failed:` and the following prefixes.
   - `CI Drift:` CI config was changed illegally. Revert or update `ops/ci` scripts and workflow config.
   - `Docs Drift:` Docs are missing key terms. Restore them.
   - `SOT Drift:` SOT is missing/ambiguous or evidence is incomplete.
     - `... sot_missing: ...` No valid SOT candidate found.
     - `... sot_ambiguous: ...` Duplicate candidates exist for the same PR number.
2. **Fix**:
   - If SOT missing: add/restore `docs/pr/PR-<digits>-*.md`.
   - If SOT ambiguous: keep **exactly one** file for that PR number (merge/delete duplicates).
   - If evidence missing: update the selected SOT so it reflects reality (e.g., sync commit SHA / include required keywords).
3. **Verify**:
   - Run `go test ./...`.
   - Run `nix run .#prverify`.
   - Ensure it passes (Green).

## Non-Goals
- Expanding drift check to arbitrary files without a clear policy.
- Parsing every file in the repo (performance).
