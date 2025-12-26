# Weekly Dogfood Reports

This directory contains the weekly usage reports and audit trails for Veil-RS dogfooding.

## Rules & Configuration (Phase 13)

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
```
This ensures the directory structure is preserved for audit purposes.

### 4. Local Reproduction
To simulate the CI run locally (ensuring your environment matches `Asia/Tokyo` for ID calculation):

```bash
# Run the dogfood loop (generates artifacts in docs/dogfood/)
nix run .#cockpit -- dogfood weekly

# Note: This will calculate WEEK_ID based on your local time (converted to Tokyo rules internally).
# If you are testing a specific week logic, ensure your system clock or logic aligns.
```

### 5. Git Policy
*   `docs/dogfood/**`: **Tracked** (Audit Trail).
*   `result/dogfood/**`: **Ignored** (Raw Events/Logs).
*   `README.md`: **Tracked** (This file).

### 6. Scoring Logic (Worklist)
*   **Score** = `(Count * 10) + (Delta * 25)`
    *   `Delta` = `Count - PrevCount`. (No penalty for improvement).
*   **Tie-breaker**: Score (DESC) > Count (DESC) > ActionID (ASC) > HintKey (ASC).
