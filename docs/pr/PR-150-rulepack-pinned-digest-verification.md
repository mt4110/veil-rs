---
release: TBD
epic: Enterprise JP PII
pr: 150
status: Ready
created_at: 2026-07-11
branch: feat/rulepack-pinned-digest-verification
commit: 5094792
title: Add RulePack pinned digest verification
---

# SOT: Add RulePack Pinned Digest Verification

## SOT
- Title: Add RulePack pinned digest verification
- Status: Ready
- PR: #150

## What
- [x] Parse `[signature]` metadata from RulePack manifests.
- [x] Enforce offline `trust_model = "pinned_digests"` with `digest_algorithm = "sha256"`.
- [x] Compute a deterministic digest from pack metadata and RulePack file SHA256 values.
- [x] Fail closed for digest mismatches and unsupported trust models such as `pinned_keys` or `tofu`.
- [x] Mark only the offline RulePack signature verification Phase 7 task complete.

## Verification
- [x] `cargo test -p veil-core rules::pack` - PASS
- [x] `cargo test -p veil-core` - PASS
- [x] `cargo fmt --check` - PASS
- [x] `cargo clippy -p veil-core --all-targets -- -D warnings` - PASS
- [x] `python3 scripts/check_docs_taxonomy.py` - PASS
- [x] `git diff --check` - PASS

## Evidence
- Built-in default RulePacks keep `signature.enabled = false`, so existing bundled behavior remains unchanged.
- Custom/offline RulePacks with `signature.enabled = true` or `signature.required = true` must match one pinned digest.
- Remote RulePack update flow remains separate and unimplemented.

## Non-goals
- [x] Do not implement public-key signature verification.
- [x] Do not implement TOFU.
- [x] Do not implement RulePack update/download flow.
- [x] Do not change remote JSON rule fetching.

## Rollback
- Revert this PR as a unit to remove pinned digest enforcement and roadmap status updates.
