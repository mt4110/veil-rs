# Runbook: Exception Management v1

## 1. Core Principles
- **Centralization**: All exceptions must be registered in the **Exception Registry** (`ops/exceptions.toml`). Scattered ignore comments are deprecated.
- **Accountability**: Every exception must have an **Owner**, a **Reason**, and an **Audit Trail**.
- **Temporality**: Exceptions should be temporary. **Expiry** (`expires_at`) is mandatory unless specific "Perpetual Exception" criteria are met.

## 2. Exception Schema (v1)

Exceptions are defined in `ops/exceptions.toml` using the `[[exception]]` array-of-tables format.

### Mandaory Fields
- **id**: Unique identifier (Format: `EX-YYYYMMDD-###`).
- **rule**: The guardrail rule identifier being suppressed.
- **scope**: The scope of the exception (see Scope Grammar).
- **reason**: Clear explanation of why this is a False Positive or acceptable risk.
- **owner**: Responsible individual (e.g., `@username`).
- **created_at**: Date of creation (`YYYY-MM-DD`).
- **audit**: Array of strings documenting the audit history (e.g., Ticket URLs, decision logs).

### Optional Fields
- **expires_at**: Expiry date (`YYYY-MM-DD`). Mandatory unless justification is provided in `reason` (e.g., "Vendor vendor-lock file").

### Scope Grammar
v1 supports exactly **two** scope types:
1.  **`path:<glob>`**: Matches file paths.
    - Example: `path:docs/**`, `path:src/legacy/*.rs`
2.  **`fingerprint:<sha256>`**: Matches specific finding fingerprints (if supported by the scanner).
    - Example: `fingerprint:a1b2c3...`

## 3. Workflows

### Adding an Exception
1.  Identify the False Positive or Blocker.
2.  Check if a Fix (code change) is possible.
3.  If not, add an entry to `ops/exceptions.toml`.
    - Generate ID: `EX-<Today>-<Seq>`.
    - Set `expires_at` (Default: 30 days from now).
4.  Run `nix run .#prverify` to confirm it passes.
5.  Commit with reason.

### Updating/Removing an Exception
- **Expired**: Fail behavior will trigger.
    - **Fix**: Resolve the issue (remove exception) OR Extend expiry (with new audit entry).
- **Stale**: Remove exceptions that no longer match any findings.

## 4. Audit & Review
- `prverify` acts as the automated auditor.
- Exceptions without valid schema or expired dates will cause `prverify` to FAIL.
