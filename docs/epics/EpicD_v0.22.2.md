# Epic D (v0.22.2): Guardrail Operations & Exceptions

**Status**: Planning
**Owner**: @mt4110

## 0. Mission
Establish a sustainable operation model for Guardrails to prevent "exception spaghetti" and clarify False Positive / False Negative handling.

## 1. Hard Constraints
- **Green verification**: `nix run .#prverify` must strictly pass on every PR and locally.
- **Deterministic Failure**: Output must be deterministic (stable ordering/wording) and provide **1-scroll recovery** (Reason -> Fix -> Next).
- **Consistency**: Docs, Runbooks, and Implementation must always match.
- **Evidence**: Single final PASS evidence (aggregated log) per PR.
- **Merge Policy**: Create a merge commit (preserve SHA stability).
- **SOT**: Code changes require a Source of Truth document.

## 2. Scope

### In Scope
- **Exception Governance**: Strong rules for reason, expiry, audit trails, and operation flows.
- **Target Extensions**: Limited to 1-2 specific extensions (Registry, Expiry).
- **Operator-first UX**: maintain "quiet when healthy" and "actionable when failing".

> [!NOTE]
> This epic introduces **Exception Registry v1** in **PR42+**. It does not change existing `.driftignore` behavior in **PR41**.

### Out of Scope
- Unrelated feature additions.
- Proliferation of exceptions without governance (must have reason + audit).
- Excessive target expansion (strictly limited scope).
- Automated exception extension/generation (prevents rot).

3.  **Target Selection**
    1.  **Target-1 (Required): Exception Registry v1 (PR42)**
        -   Centralized ledger (`ops/exceptions.toml`).
        -   Explicit schema validation (id, rule, scope, reason, owner, audit).
        -   *Note: Expiry is NOT enforced in PR42.*
    2.  **Target-2 (Recommended): Expiry Enforcement (PR43+)**
        -   Fail on expired exceptions.
        -   1-scroll recovery UX.

## 4. Acceptance Criteria
- [ ] `prverify` is Green.
- [ ] Failure output is deterministic and actionable (1-scroll).
- [ ] Docs/Runbook match the implementation.
- [ ] Exceptions are centralized in the registry with mandatory fields.
- [ ] Expiry is enforced (or strictly governed).
