---
release: TBD
epic: A
pr: TBD
status: Ready
created_at: 2026-07-09
branch: feat/lsp-partial-mask-code-action
commit: TBD
title: Add LSP partial mask code action
---

## SOT
- Title: Add LSP partial mask code action
- Status: Ready
- PR: TBD

## What
- [x] Add `partial_mask` to the LSP diagnostic action metadata.
- [x] Expose a `Partial mask` quick fix alongside `Mask value` when the diagnostic supports it.
- [x] Apply the existing core `MaskMode::Partial` masking logic to the diagnostic range instead of inventing a separate LSP masking path.
- [x] Keep inline ignore unavailable for JSON while still offering full and partial masking.
- [x] Update the LSP design note so the documented quick fixes match the implementation.

## Verification
- [x] `cargo clippy --workspace --all-targets --all-features -- -D warnings` - PASS
- [x] `git diff --check` - PASS
- [x] `cargo test -p veil-lsp --all-targets` - PASS (`23` tests)

## Evidence
- Local command output observed on 2026-07-09 in `feat/lsp-partial-mask-code-action`.
- CI evidence will be the GitHub pull request checks for this branch.

## Non-goals
- [x] Does not change scanner finding ranges; CodeAction continues to trust the diagnostic range as the SOT.
- [x] Does not add partial masking to languages or contexts where no diagnostic action advertises it.
- [x] Does not change core partial masking semantics.

## Rollback
- Revert this PR as a unit, or remove the generated SOT file if the PR is abandoned.
