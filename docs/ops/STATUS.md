
# STATUS (Single Source of Truth)

This file is the *only* canonical progress board for ops phases (S11+).
All PRs starting from S11-00 MUST update this file (at least the % and "Last Updated" line).

## Rules (Non-Negotiable)
- Single board: do not create other progress trackers.
- Deterministic edits:
  - Keep table row order fixed (S11..S15).
  - Update ONLY the "%", "Current", and "Last Updated" fields unless adding a brand-new phase.
- One PR, one truth:
  - If a PR touches phase scope, it MUST update this board in the same PR.
- Percent is human-reported but must be consistent:
  - 0% = not started
  - 1-98% = in progress
  - 99% = Review (DoD: PR open, CI pass)
  - 100% = merged to main (or otherwise declared complete with evidence)

## Milestones (S11..S15)

| Phase    | Goal (One-liner)                                    | Progress      | Current (This PR)            |
| -------- | --------------------------------------------------- | ------------- | ---------------------------- |
| S11-00   | Kickoff: pin roadmap/progress board in STATUS.md    | 100% (Merged) | -                            |
| S11-01   | Enforce STATUS.md update (forget -> fail)           | 100% (Merged) | -                            |
| S11-02   | SOT guidance truth (stopless design)                | 100% (Merged) | -                            |
| S11-03   | Review Bundle Go Hardening (deterministic)          | 100% (Merged) | -                            |
| S11-04   | Hermetic Determinism Tests                          | 100% (Merged) | -                            |
| S11-05   | Closeout: mark S11-03/04 merged + fresh prverify    | 100% (Merged) | -                            |
| S12-00   | TBD                                                 | 100% (Merged) | -                            |
| S12-01   | TBD                                                 | 100% (Merged) | -                            |
| S12-02   | Closeout + ritual spec (zsh-safe)                   | 100% (Merged) | -                            |
| S12-03   | Strict Ritual Capsule (commit+prverify+strict)      | 100% (Merged) | -                            |
| S12-04   | CI repro ritual capsule (prkit ci-repro)            | 100% (Merged) | -                            |
| S12-05   | CI repro cleanup (runner alignment)                 | 100% (Merged) | -                            |
| S12-05.5 | speed up local prverify (safe parallel + caching)   | 100% (Merged) | -                            |
| S12-05.6 | prverify stopless hardening (no os.Exit + bugfixes) | 1% (WIP)      | S12-05.6: stopless hardening |
| S13      | TBD                                                 | 0%            | -                            |
| S14      | TBD                                                 | 0%            | -                            |
| S15      | TBD                                                 | 0%            | -                            |

## Last Updated
- Date: 2026-02-24 (+0900)
- By: @mt4110
- Agent: @antigravity
- Evidence: docs/ops/S12-05-6_PLAN.md

## Update Checklist (for every PR)
- [x] Update the Progress % and Current (This PR) row(s)
- [x] Update "Last Updated" (Date, By, Evidence)
- [x] Ensure row order is unchanged
