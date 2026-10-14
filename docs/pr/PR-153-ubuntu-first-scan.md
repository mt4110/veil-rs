---
pr: 153
status: Draft
created_at: 2026-10-14
branch: docs/ubuntu-first-scan
title: Ubuntu first scan and shared pre-commit SOT guard
---

# PR #153: Ubuntu first scan and pre-commit SOT guard

## SOT

README and the CLI tutorial describe the published 0.21.0-rc.1 trial and the first own-code scan on Ubuntu x86_64. The SOT Guard policy remains unchanged: changes outside docs/ and .github/ require a changed PR-specific SOT. README is included in that requirement.

## What

- Put downloading, checksum verification, one-file scanning, exit-code interpretation, and repeat use before optional synthetic examples.
- Correct unpublished-RC wording and the invalid `--fail-on-findings 0` example.
- Preserve macOS ad-hoc limitations and distinguish updated repository docs from existing ZIP contents.
- Share the SOT checker between hosted CI and the existing native pre-commit hook, preserving the Markdown formatter guard.
- Install the hook through an absolute repository-local hooks path so older sibling worktrees are protected without source/index edits.

## Verification

The published Linux ZIP was downloaded and checksum-verified on Ubuntu. Isolated Python inputs returned 0, 1 after adding a synthetic key, and 2 for nonexistent input; source files stayed unchanged and the key stayed masked.

Guard regression tests cover the original README-only failure, docs-only exemption, staged/unstaged/deleted SOT, prior branch commits, CI snapshots, and missing base refs. All 9 SOT regression tests passed, including actual Git commit rejection without SOT and acceptance with staged SOT, and rejection by the retained Markdown guard. The installed repository hook also rejected an actual commit on the original PR head without changing HEAD. All 94 Python script tests, architecture, release, docs taxonomy, actionlint, shell syntax, and whitespace checks passed locally.

## Evidence

Original missing-SOT failure: [SOT Guard](https://github.com/mt4110/veil-rs/actions/runs/38490548505).
The original PR head is `e2b8a7d1aa4e15fd69011b5b276719b5f963d7ae`. Hosted checks for the updated head must be verified after push; prior-head results are not proof for this change.

## Non-goals

No merge, Release changes, publication, automatic SOT content generation, or edits to the preserved owner checkout. Hooks do not replace the full Rust CI matrix or establish user adoption.
