# Runbook: Exception Registry Operations

## Overview
This runbook covers the operation of the **Exception Registry** (`ops/exceptions.toml`).
Use the `veil exceptions` command family to inspect and audit exceptions.

## Common Operations

### 1. Listing Exceptions
To view all registered exceptions and their status:
```bash
veil exceptions list
# Filter by status
veil exceptions list --status expired
veil exceptions list --status expiring_soon
```

### 2. Inspecting a Specific Exception
To view details (reason, audit trail, full text) of a specific ID:
```bash
veil exceptions show <id>
```

## Troubleshooting Failures

### Case 1: "Exception Expired"
If `prverify` fails with an expiry error, the output will look like this:
```
Registry validation failed (1 errors): (utc_today=2026-02-08)
- EX-20260101-01: expired (expires=2026-01-01, now=2026-02-08, status=expired)
Fix:    Renew expiry or remove exception (see runbook: docs/runbook/exception-registry.md)
Next:   nix run .#veil -- exceptions list --status expired
```

**Resolution Steps:**
1. **Identify**: Run the suggested `Next` command:
   ```bash
   nix run .#veil -- exceptions list --status expired
   ```
2. **Fix Option A (Renew)**: If the exception is still valid:
   - Edit `ops/exceptions.toml`.
   - Update `expires_at` to a future date (YYYY-MM-DD).
   - Append a new entry to `audit` explaining why it was extended.
3. **Fix Option B (Remove)**: If the exception is no longer needed:
   - Remove the `[[exception]]` block from `ops/exceptions.toml`.
4. **Verify**: Run `nix run .#prverify`.

### Case 2: "Invalid Format"
If `prverify` fails with validation errors (e.g., missing fields, bad scope):
```
Registry validation failed (1 errors):
- EX-INVALID-01: missing required field 'owner'
Fix:    Correct the invalid entries in ops/exceptions.toml
Next:   nix run .#veil -- exceptions list
```

1. **Check**: The error message usually points to the specific field (e.g., `invalid scope`).
2. **Fix**: Edit `ops/exceptions.toml` to match the schema (see `docs/runbook/exceptions_v1.md`).
3. **Verify**: Run `nix run .#prverify`.

## Maintenance
- **Expiring Soon**: Periodically run `veil exceptions list --status expiring_soon` to catch issues before they break CI.

## Operation Rules

### Renewal Responsibility
- The owner of the exception (`owner` field) is responsible for renewing or removing it before it expires.
- Expired exceptions block `prverify` and must be resolved immediately.

### ID Naming Convention
- IDs should be unique and descriptive.
- Format: `EX-<YYYYMMDD>-<SEQ>` (Recommended) or `EX-<FEATURE>-<SEQ>`.
- Examples: `EX-20260208-01`, `EX-SQLX-01`.


