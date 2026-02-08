# Runbook: Exception Management v1

## Status / Scope
- **Target**: This runbook governs the upcoming **Exception Registry v1** (`ops/exceptions.toml`).
- **Enforcement Timeline**:
    - **PR41**: Existing `.driftignore` behavior maintained (Registry is documentation-only).
    - **PR42**: Registry presence + Schema/Format validation.
    - **PR43+** (Active): Expiry enforcement ACTIVE (Expired items FAIL).

## 1. Core Principles
- **Centralization**: For **Exception Registry v1** (PR42+), all exceptions must be registered in the **Exception Registry** (`ops/exceptions.toml`). Scattered ignore comments are deprecated.
- **Accountability**: Every exception must have an **Owner**, a **Reason**, and an **Audit Trail**.
- **Temporality**: Exceptions should be temporary. **Expiry** (`expires_at`) is mandatory for policy (PR43+ enforcement). Use "Perpetual Exception" criteria for long-term overrides.

## 2. Exception Schema (v1)

Exceptions are defined in `ops/exceptions.toml` using the `[[exception]]` array-of-tables format.

### Mandatory Fields
- **id**: Unique identifier (Format: `EX-YYYYMMDD-###`).
- **rule**: The guardrail rule identifier being suppressed.
- **scope**: The scope of the exception (see Scope Grammar).
- **reason**: Clear explanation of why this is a False Positive or acceptable risk.
- **owner**: Responsible individual (e.g., `@username`).
- **created_at**: Date of creation (`YYYY-MM-DD`).
- **audit**: Array of strings documenting the audit history (e.g., Ticket URLs, decision logs).

### Optional Fields (Strict Policy)
- **expires_at**: Expiry date (`YYYY-MM-DD`).
- **expires_at**: Expiry date (`YYYY-MM-DD`).
    - **Policy**: Recommended (should exist).
    - **Enforcement (PR43)**:
        - If present: **Enforced** (must be today or later in UTC; expired only when utc_today > expires_at).
        - If missing: **Allowed** (PASS in PR43).
    - **Long-term**: Perpetual exceptions (no expiry) require audit trails and periodic review.

### Scope Grammar
v1 supports exactly **two** scope types:
1.  **`path:<glob>`**: Matches file paths.
    - Example: `path:docs/**`, `path:src/legacy/*.rs`
2.  **`fingerprint:<sha256>`**: Matches specific finding fingerprints (if supported by the scanner).
    - Example: `fingerprint:a1b2c3...`

## 3. Expiry Semantics (Deterministic)
- **Format**: `YYYY-MM-DD` (ISO 8601).
- **Inclusive**: `expires_at` date is valid (exception active).
- **Expired**: When `UTC_Today > expires_at`.
- **Timezone**: All dates are UTC.

## 4. False Negative (FN) Policy
- **Prohibited**: Do NOT use exceptions to "fix" False Negatives (missing detection).
- **Action**: FNs require a test case (fixture) and a rule improvement to detect the issue.

## 5. Workflows

### Adding an Exception
1.  Identify the False Positive or Blocker.
2.  Check if a Fix (code change) is possible.
3.  If not, add an entry to `ops/exceptions.toml`.
    - Generate ID: `EX-<Today>-<Seq>`.
    - Set `expires_at` (Default: 30 days from now).
4.  Run `nix run .#prverify` to confirm it passes.
5.  Commit with reason.

### Updating/Removing an Exception
- **Expired**: Fail behavior is **ACTIVE** (PR43+).
    - **Fix**: Resolve the issue (remove exception) OR Extend expiry (with new audit entry).
- **Stale**: Remove exceptions that no longer match any findings.

## 6. Audit & Review
- `prverify` acts as the automated auditor.
- **PR42**: schema/date-format violations FAIL.
- **PR43+** (Active): expiry enforcement (UTC today > expires_at) FAIL.

> [!NOTE]
> `ops/exceptions.toml` is valid even if empty (0 exceptions).
