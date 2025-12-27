# Weekly Dogfood Operations (Phase 15)

This document defines the operational rules for the "Weekly Dogfood" loop.
Goal: **Continuous Tiny Improvements (Evidence → Insight → Tiny Fix)**.

## 1. Golden Rules
1.  **Silence Forbidden**: Every week MUST result in a `FIXLOG.md` entry. Even if nothing is fixed, a "NOOP" entry proving observation is required.
2.  **Top 1 Strategy**: Identify the single most critical signal. Fix *only* that (max Top 3 if strict tiny fixes).
3.  **Tiny Fix Only**: 
    *   No major refactors.
    *   No new dependencies (unless critical).
    *   Must be completable in <1 hour.
4.  **Evidence is King**: All decisions must link to `SUMMARY.md`, `signals_v1.json`, or `ACTIONS.md`.

## 2. Decision Criteria (Priority)
When selecting the "Top 1" fix, use this priority order:

1.  **Safety / Security** (Credentials, Drift, Public Exposure)
2.  **Integrity** (CI Broken, Contract Violation, Missing Artifacts)
3.  **Noise** (False Positives, Flaky tests)
4.  **UX** (Console output, Readability)

## 3. Workflow
1.  **Monday**: CI generates artifacts in `docs/dogfood/YYYY-Www-Tokyo/`.
2.  **Review**: Operator checks `SUMMARY.md` and `ACTIONS.md`.
3.  **Plan**: Select Top 1 Action. 
4.  **Fix**: Apply the tiny fix.
5.  **Log**: Update `docs/dogfood/YYYY-Www-Tokyo/FIXLOG.md`.
6.  **Commit**: Commit Fix + FIXLOG.

## 4. Templates
See [FIXLOG Template](./templates/FIXLOG.md).
