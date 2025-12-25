# Weekly Dogfood Reports

This directory contains the weekly usage reports and audit trails for Veil-RS dogfooding.

## Rules & Configuration (Phase 12)

### 1. Exclusion Rules
- **`dogfood.*` operations** are **excluded** from the Worklist / Top 3 suggestions.
  - **Rationale**: Infrastructure failures or testing noise in the dogfood loop itself should not clutter the user's improvement list.
  - **Implementation**: `internal/cockpit/dogfood.go` expressly filters these out during aggregation.
- They are **included** in the "Total Failure Events" metrics (`counts_by_reason`) to maintain an accurate audit trail of system reliability.

### 2. Output Contract
The `cockpit dogfood weekly` command ensures the following files are generated for each week.
Directory naming convention: `docs/dogfood/{YYYY-Www}-Tokyo/` (e.g., `2025-W52-Tokyo`).

Files:
- `metrics_v1.json`: Audit log aggregation.
- `worklist.json`: Top 3 improvement candidates.
- `report.md`: Human-readable report summary.
- `scorecard.txt`: OSSF Scorecard result.

### 3. Git Policy
- `docs/dogfood/**` is tracked (Audit Trail).
- `result/dogfood/**` is ignored (Raw Events).
- This `README.md` is tracked.

### 4. Scoring logic (Worklist)
- **Score** = `(Count * 10) + (Delta * 25)`
    - `Delta` = `Count - PrevCount`. If `Delta < 0`, `25 * 0` is used (no penalty for improvement, but no bonus).
- **Tie-breaker**:
    1. Score (DESC)
    2. Count (DESC)
    3. ActionID (ASC)
    4. HintKey (ASC)

Reference: [Phase 12 Spec](../ai/PHASE12_SPEC_WEEKLY_DOGFOOD.md)
