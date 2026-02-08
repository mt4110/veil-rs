# Runbook: Exception Management v1

## Status / Scope
- **Target**: This runbook governs the upcoming **Exception Registry v1** (`ops/exceptions.toml`) introduced in PR42+.
- **Current**: Existing drift-check exceptions remain governed by `.driftignore` and `docs/guardrails/drift.md` until fully migrated.

## 1. Core Principles
- **Centralization**: For **Exception Registry v1** (PR42+), all exceptions must be registered in the **Exception Registry** (`ops/exceptions.toml`). Scattered ignore comments are deprecated.
- **Accountability**: Every exception must have an **Owner**, a **Reason**, and an **Audit Trail**.
- **Temporality**: Exceptions should be temporary. **Expiry** (`expires_at`) is mandatory by default. Use "Perpetual Exception" criteria for long-term overrides.

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
    - **Policy**: SHOULD exist. Omission requires meeting **Perpetual Exception** criteria:
        1. **Narrow Scope**: Must not wildcard broadly (e.g., no `path:**`).
        2. **Durable Audit**: Must link to a permanent decision record (e.g., Architecture Decision Record).
        3. **Explicit Reason**: Must state why expiry is impossible (e.g., "Vendor lock file format").
        4. **Periodic Review**: Owner must review manually (e.g., quarterly).

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
- **Expired**: Fail behavior will trigger.
    - **Fix**: Resolve the issue (remove exception) OR Extend expiry (with new audit entry).
- **Stale**: Remove exceptions that no longer match any findings.

## 6. Audit & Review
- `prverify` acts as the automated auditor.
- Exceptions without valid schema or expired dates will cause `prverify` to FAIL.
