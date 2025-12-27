# Phase 16 / #M1 Signal Definition, Normalization, and Recurrence Detection
**Sync Document (Implementation Prerequisites)**

## 0. Positioning
This document solidifies the handling of **Signals** in Phase 16 / #M1 as an "implementable specification".
- Design philosophy: **Fixed**
- Implementation details (Code structure / IO): **Flexible**
- **Prohibitions**: **Must be strictly observed**

## 1. Objective of #M1 (Why)
The goal is to elevate weekly dogfood signals from:
- One-off notices
- Transient logs
To:
- **Structural Elements** (Trackable entities) that represent "Recurring Signs".

**Important**: At this stage, **NO Automatic Fixes** and **NO Automatic Judgments** are performed.
The scope is strictly **Observation, Classification, and Recording**.

## 2. Input Scope (Signal Sources)
Signals are extracted from ALL of the following:
- Dogfood JSON output
- stdout / stderr text output
- FIXLOG
- CI Logs
*Raw logs must be normalized before use.*

## 3. Evidence Scope
### MUST (Conditions for Signal formation)
- Run Start/End Info
- WEEK_ID
- Commit Hash
- Difference from Last Week
    - Increase / Decrease / Unchanged
- Progress Judgment
    - Improvement / Regress / Flat
- Execution Time
    - Total
    - Per Step
- Failure Cause
    - First failed step
    - Exit code

### OPTIONAL (Auxiliary Info)
- Existence of Artifacts (`dist/`, `docs/`)
- Environment Info (OS/Arch, Toolchain, Nix Store)

## 4. Signal Granularity
- **1 Signal = 1 Category**
- Multiple rules in one file is allowed.
- Mixing different categories in one file is **Prohibited**.
    - Example: Entropy signals in `entropy` file only. Scope signals in `scope` file only.

## 5. Signal Normalization
### MUST Keys (Minimum Complete Unit)
- `category`
- `cause_tag` (based on rule_id)
- `severity`
- `week_id`
- `commit_sha`

### SHOULD Keys
- `source` (json / text / fixlog / ci)
- `artifact_ref` (Link to evidence)

### Normalized Signal Example
```json
{
  "week_id": "2025-W12",
  "commit_sha": "abc123",
  "category": "entropy",
  "cause_tag": "R001",
  "severity": "improve",
  "source": ["dogfood-json", "ci-log"],
  "artifact_ref": "docs/dogfood/2025-W12/entropy.json"
}
```

## 6. Handling "Heavy" Improvement Proposals
- **Threshold**: 5 minutes
- **Behavior when exceeded**:
    - Upgrade severity to `warn`
    - Skip detailed proposal generation (Lightweight mode)
    - Continue run
    - Do NOT treat as NOOP
    - **Do NOT abort run** (Prohibited in Phase 16)

## 7. Duplicate Handling
If the same content appears in multiple sources:
- Merge into **1 Signal** during normalization.
- Keep sources as an array.
- **Double counting is prohibited.**

## 8. Recurrence Detection
### Conditions
Same `category` + `cause_tag` matches either:
- **3 consecutive weeks**
- **3 times in the last 4 weeks** (Loose condition)

### Result
- Issue `recurring:<category>:<cause_tag>`
- Record in Ledger
- Display as "Ongoing" in Weekly Report

## 9. Storage
- Weekly Evidence: `docs/dogfood/YYYY-WXX/`
- Cross-sectional Ledger: `docs/dogfood/recurring_signals.json`

## 10. Safety Rails / MUST NOT (Prohibited)
In Phase 16 / #M1, absolutely **DO NOT**:
- Auto-fail
- Auto-create Issues
- Auto-create PRs
- Auto-label GitHub issues
- Auto-nudge refactoring

*Judgments and responses must be left to humans.*

## 11. Human Judgment (Out of Scope)
- Decisions on fixing recurring signals, deferring them, or ignoring them are human tasks.
- Decisions are recorded in `FIXLOG` or `Decision Log`.

## 12. Summary for Implementer
- #M1 is the **Structuring Phase**.
- Automation stops at **Observation, Classification, Recording**.
- Judgment/Action remains with humans.
- **Quiet Strength** is the priority. 
