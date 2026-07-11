---
release: TBD
epic: UI
pr: TBD
status: Draft
created_at: 2026-07-11
branch: feat/ui-print-report-export
commit: TBD
title: Add browser print path for audit reports
---

# SOT: Add Browser Print Path For Audit Reports

## SOT
- Title: Add browser print path for audit reports
- Status: Draft
- PR: TBD

## What
- [x] Add a `Print Report` action beside Evidence ZIP export in Local Audit UI scan results.
- [x] Use browser print as the initial PDF export path instead of adding a headless renderer or native PDF dependency.
- [x] Add print CSS that hides navigation and scan controls while preserving scan summary and finding tables.
- [x] Mark the PDF export path and browser print design tasks complete while keeping native `--format pdf` future-gated.

## Verification
- [x] `npm --prefix crates/veil-pro/frontend run check` - PASS
- [x] `npm --prefix crates/veil-pro/frontend run build` - PASS
- [x] `python3 scripts/check_docs_taxonomy.py` - PASS
- [x] `git diff --check` - PASS

## Evidence
- The UI export path remains local-only and depends on `window.print()`.
- The print stylesheet is CSS-only and does not add network, renderer, or file-system permissions.

## Non-goals
- [x] Do not add CLI `--format pdf`.
- [x] Do not add a headless browser or server-side PDF renderer.
- [x] Do not change Evidence ZIP generation.

## Rollback
- Revert this PR as a unit to remove the print button, print CSS, and roadmap status updates.
