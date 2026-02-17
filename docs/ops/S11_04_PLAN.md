# S11-04 Plan: Hermetic Determinism Tests

## Goal
Eliminate flaky CI failures in `TestCreate_Determinism` caused by missing `origin/main` or shallow checkout depth by making `reviewbundle` tests fully hermetic.

## Problem
Currently, `reviewbundle` tests invoke `git format-patch` against `main` or specific refs. In CI (and some local environments), these refs may be missing or shallow, causing `exit status 128` and test failure.

## Solution
Refactor `determinism_test.go` to use a **synthetic, in-process git repository** created in `t.TempDir()`.
1. Initialize a bare-bones git repo in `TempDir`.
2. Create a `main` branch and a feature commit.
3. Use this local repo path and refs in the `Contract`.
4. Verify `create` -> `verify` loop is deterministic and passes without external network/ref dependencies.

## Tasks (C0-C2)
- [ ] **S11-04 C0**: `testutil_gitrepo.go` - Git Helper (Init, Commit, Tag)
- [ ] **S11-04 C1**: `determinism_test.go` - Refactor to use synthetic repo
- [ ] **S11-04 C2**: Docs - Update STATUS and generate Evidence
