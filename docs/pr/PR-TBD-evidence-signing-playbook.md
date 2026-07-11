---
release: TBD
epic: Enterprise JP PII
pr: TBD
status: Draft
created_at: 2026-07-11
branch: feat/evidence-signing-playbook
commit: TBD
title: Add evidence signing playbook
---

# SOT: Add Evidence Signing Playbook

## SOT
- Title: Add evidence signing playbook
- Status: Draft
- PR: TBD

## What
- [x] Add a v1 playbook for signing Evidence approval records outside `evidence.zip`.
- [x] Bind signing to the existing `run_meta.json` raw bytes SHA256 external anchor.
- [x] Document required inputs, signing boundary, procedure, failure handling, and retention.
- [x] Mark only the Evidence signing playbook Phase 7 task complete.

## Verification
- [x] `python3 scripts/check_docs_taxonomy.py` - PASS
- [x] `git diff --check` - PASS
- [x] `rg -n 'tamper-proof|signature files inside ...|upload source|upload.*PII' ...` - reviewed wording and confirmed only negative/safety contexts

## Evidence
- The playbook does not change the Evidence ZIP v1 entry contract.
- The playbook does not require network access or a specific signing vendor.
- The playbook keeps Remote RulePack signatures as a separate Phase 7 trust boundary.

## Non-goals
- [x] Do not implement signature verification code.
- [x] Do not add signature files inside `evidence.zip`.
- [x] Do not implement Remote RulePack signature verification.
- [x] Do not implement RulePack update flow.

## Rollback
- Revert this PR as a unit to remove the playbook and design status update.
