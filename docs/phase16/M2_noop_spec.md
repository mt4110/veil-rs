# Phase 16 #M2: NOOP Specification - Observed Invariant

**Goal**: Establish "NOOP" as a quantifiable measure of stability (**Observed Invariant**), distinct from "laziness" or "failure".

## Core Concept
**Observed Invariant (NOOP)**:
A system state where no structural change is observed between consecutive weekly runs.
NOOP does not imply inactivity. It represents a measured stability of the system under continuous observation.

## Specification

### 1. Definition of NOOP
A week is considered **NOOP** if there is **no delta** in normalized signals compared to the previous week.
*   **Implies**: `Signals[Current Week] == Signals[Previous Week]` (Set equality).
*   **Pure NOOP**: `len(Signals) == 0` (Empty set). A subset of NOOP.

### 2. Granularity
NOOP is tracked at the **Run Level** (Weekly).
*   The entire weekly execution is classified as either "Changed" or "NOOP".

### 3. Stability Metrics
*   **Consecutive NOOPs** (Primary): A counter that increments for every consecutive NOOP week, resets on change.
*   **NOOP Rate** (Secondary): Percentage of NOOP runs over total observed runs (calculated metrics).

### 4. Storage & Ledger
Data is stored in a dedicated **Stability Ledger**, separate from recurrence data.
*   File: `docs/dogfood/stability_ledger.json`

**Schema (Draft):**
```json
{
  "v": 1,
  "runs": [
    {
      "week_id": "2025-W02",
      "result": "NOOP",          // or "CHANGED"
      "is_pure": false,          // true if empty
      "consecutive_count": 5,    // 5th consecutive week
      "delta_summary": ""        // Optional: summary of change if CHANGED
    }
  ],
  "current_streak": 5
}
```

### 5. Implementation Rules
*   **Input**: Current Normalized Signals (`signals_v1.json`) and Previous Normalized Signals.
*   **Logic**:
    1.  Load `stability_ledger.json`.
    2.  Load Previous Signals (from disk or memory).
    3.  Compare Current vs Previous.
    4.  Determine `NOOP` status and update Streak.
    5.  Append entry to Ledger.
    6.  Save Ledger.
*   **Prohibitions**:
    *   NO Fail on change.
    *   NO Alert on change/NOOP.
    *   NO Auto-Issue/PR.

## Implementation Note
#M2 treats NOOP as an observed invariant, not as an absence of work.
The implementation must record stability without triggering any action.
This data is for human interpretation and future phase design only.
