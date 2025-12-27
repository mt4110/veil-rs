# Phase 16 #M2: NOOP Design Brief

**Goal**: Define "NOOP" not as "nothing happened" but as "**Observed Invariant State**" (System Stability).

## Premise (Fixed)
*   Signal / Recurrence is implemented (#M1).
*   Dogfood runs weekly.
*   **Safety Rails**: No auto-judgment, no auto-fix.

## Core Philosophy
*   NOOP is not "lazy" (怠慢).
*   NOOP is a form of "stability" (安定).
*   NOOP teaches us "where it breaks next" (次に壊れる場所).

## Design Q List (To Be Decided)

### Q1. Definition of NOOP
What state constitutes a "NOOP" for a given week?
*   [ ] **Option A**: 0 Signals generated.
*   [ ] **Option B**: Signals exist, but no *active recurring* signals (all transient).
*   [ ] **Option C**: No *delta* from previous week (Same signals as last week).
*   [ ] **Option D**: Explicit "Pass" events only?

### Q2. Granularity
At what level do we count/track NOOP?
*   [ ] **Run Level**: The entire weekly dogfood run is a NOOP.
*   [ ] **Category Level**: "Audit" is NOOP, but "Entropy" had signals.
*   [ ] **Signal Level**: Specific signal did not recur (NOOP for that signal?).

### Q3. Metrics
What numbers do we want to see?
*   [ ] **NOOP Rate**: % of runs that are NOOP over time.
*   [ ] **Consecutive NOOPs**: How many weeks has it been stable? (Stability Counter).
*   [ ] **Transitions**: Frequency of Stability -> Instability (Signal).

### Q4. Storage
Where does this data live?
*   [ ] **Weekly Artifact**: `metrics_v1.json` field? Or new `noop_v1.json`?
*   [ ] **Ledger**: Update `recurring_signals.json`? Or new `stability_ledger.json`?

### Q5. Prohibitions (Confirmed)
*   NO Fail.
*   NO Alert.
*   NO Auto-Issue/PR.

---

## Discussion
Please select options or provide guidance for Q1-Q4.
Legacy "Metrics" tracked `counts_by_reason`. If `counts_by_reason` is empty (or only contains "pass"?), is that NOOP?
Phase 16 Signal Norm puts everything into `signals_v1.json`.
If `signals_v1.json` is empty `[]`, is that NOOP?
