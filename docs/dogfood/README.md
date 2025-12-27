# Weekly Dogfood Reports

This directory contains the weekly usage reports and audit trails for Veil-RS dogfooding.

## Phase 15 Operations (Tiny Fix Loop)

- Rulebook: `docs/dogfood/OPS.md`
- FIXLOG template: `docs/dogfood/templates/FIXLOG.md`

Every week, write a short log at:
`docs/dogfood/<WEEK>/FIXLOG.md` (NOOP is allowed, silence is not).

## Rules & Configuration (Legacy: Phase 12–14)

### 1. The "Unbreakable Ritual" (Immutability)
*   **Silence is Forbidden**: Every week MUST produce an evidence commit, even if the system is completely broken.
*   **Strict Scope**: The weekly automation branch `automation/dogfood/{WEEK_ID}` MUST ONLY contain changes within `docs/dogfood/{WEEK_ID}-Tokyo/`. ANY other change (e.g. `go.mod`, unrelated docs) causes immediate workflow failure.

### 2. Output Contract
*   Directory: `docs/dogfood/{YYYY-Www}-Tokyo/` (Derived from `Asia/Tokyo` time).
*   **Required Files** (Always present):
    1.  `metrics_v1.json`: Aggregated metrics and audit trail.
    2.  `scorecard.txt`: OSSF Scorecard output.
    3.  `summary.md`: Human-readable summary and worklist.
    4.  `run.log`: Execution logs (standard output/error).

### 3. Failure Semantics
If an artifact cannot be generated (e.g., tool crash, compilation error), a **placeholder** defined by the workflow is created to satisfy the contract.

**Placeholder Format:**
```yaml
status: failed
reason: Artifact generation failed or skipped
remediation: Check workflow logs and run.log
timestamp: ...
