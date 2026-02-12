# PR57 Task v2 — Issue #47 PR-D (git2) Remediation (restart-friendly)

## Rule: each Phase ends with a commit
If execution stops mid-way, resume from the last committed Phase.

---

## Phase 0 — footing
- [x] 0.0 repo root confirmed
- [x] 0.1 worktree clean (or stashed)
- [x] 0.2 tools exist: gh/jq/rg/cargo
- [x] 0.3 branch created
- [x] 0.4 evidence dirs created
- [x] Commit: docs scaffold

STOP IF:
- worktree cannot be made clean without uncertainty

---

## Phase 1 — PRE evidence
- [ ] 1.0 UTC set and printed
- [ ] 1.1 dependabot PRE snapshot saved (json + md)
- [ ] 1.2 cargo tree PRE saved
- [ ] 1.3 Cargo.lock PRE proof saved
- [ ] Commit: evidence pre snapshots

STOP IF:
- gh api fails (auth/permissions) → record error output into a file and stop

---

## Phase 2 — upgrade
- [ ] 2.0 locate git2 declaration line
- [ ] 2.1 Cargo.toml version -> 0.20.4 (features unchanged)
- [ ] 2.2 cargo update -p git2 --precise 0.20.4
- [ ] 2.3 cargo test -p veil-cli PASS
- [ ] Commit: deps bump

BRANCH IF:
- cargo update complains multiple versions:
  - [ ] run cargo tree -i git2
  - [ ] retry cargo update -p git2@<old> --precise 0.20.4

STOP IF:
- dependency resolution attempts broaden beyond git2 (do not run cargo update without -p)

---

## Phase 3 — workspace tests
- [ ] cargo test --workspace PASS
- [ ] If failing: minimal fix only
- [ ] Commit: only if code changed

STOP IF:
- test failure cause is unclear or requires refactor

---

## Phase 4 — POST evidence
- [ ] UTC_POST set and printed
- [ ] cargo tree POST saved
- [ ] Cargo.lock POST proof saved
- [ ] dependabot POST snapshot saved (json + md)
- [ ] Commit: evidence post snapshots

NOTE:
- dependabot may still show open alerts briefly; evidence is still valid.

---

## Phase 5 — prverify evidence
- [ ] nix run .#prverify PASS
- [ ] prverify report present under docs/evidence/prverify
- [ ] Commit: prverify evidence

STOP IF:
- prverify fails; capture the log snippet into SOT notes and stop

---

## Phase 6 — cockpit
- [ ] find cockpit command via ripgrep
- [ ] run cockpit check PASS
- [ ] record command + PASS snippet into SOT

STOP IF:
- cockpit command cannot be identified deterministically

---

## Phase 7 — SOT
- [ ] docs/pr/PR-57-issue-47-pr-d-git2-remediation.md created/updated
- [ ] evidence links listed (pre/post/prverify)
- [ ] Always Run Contract results documented
- [ ] Commit: SOT

---

## Phase 8 — final verify + push
- [ ] cargo test --workspace PASS
- [ ] nix run .#prverify PASS
- [ ] git status clean
- [ ] push branch
