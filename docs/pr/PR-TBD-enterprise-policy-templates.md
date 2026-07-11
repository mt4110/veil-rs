---
release: TBD
epic: Enterprise JP PII
pr: TBD
status: Draft
created_at: 2026-07-11
branch: feat/enterprise-policy-templates
commit: TBD
title: Add enterprise policy templates
---

# SOT: Add Enterprise Policy Templates

## SOT
- Title: Add enterprise policy templates
- Status: Draft
- PR: TBD

## What
- [x] Add organization-layer policy templates for Japanese enterprise deployments.
- [x] Document that these templates are used through `VEIL_ORG_CONFIG` and are not built-in presets.
- [x] Cover standard corporate, fintech, government, SI vendor, and log-audit deployment profiles.
- [x] Mark only the Enterprise policy templates Phase 7 task complete.

## Verification
- [x] `python3 - <<'PY' ...` - policy templates parse and contract checks passed
- [x] `VEIL_ORG_CONFIG=docs/design/enterprise_jp_pii/templates/policies/enterprise-fintech.toml cargo run -q -p veil-cli -- config dump --layer org` - PASS
- [x] `VEIL_ORG_CONFIG=docs/design/enterprise_jp_pii/templates/policies/enterprise-logs.toml cargo run -q -p veil-cli -- config dump --layer org` - PASS
- [x] `python3 scripts/check_docs_taxonomy.py` - PASS
- [x] `git diff --check` - PASS

## Evidence
- The templates use only existing config fields and rule override fields.
- The templates avoid `preset` because `CoreConfig.preset` is intentionally not implemented yet.
- The templates avoid `remote_rules_url` because signed remote RulePack delivery remains a separate Phase 7 task.

## Non-goals
- [x] Do not implement offline RulePack signature verification.
- [x] Do not implement RulePack update flow.
- [x] Do not implement `CoreConfig.preset`.
- [x] Do not add hard, non-overridable organization policy semantics.

## Rollback
- Revert this PR as a unit to remove the policy templates and design status update.
