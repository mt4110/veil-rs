# PR-56 Task List — Issue #47 PR-C (idna/url)
Slug: issue-47-prc-idna-url-remediation

## 0) Branch + files
- [x] `git switch main`
- [x] `git pull --ff-only`
- [x] `git switch -c feature/pr56-issue47-prc-idna-url-remediation`
- [x] Create files:
  - [x] docs/pr/PR-56-issue-47-prc-idna-url-remediation.md
  - [x] docs/pr/PR-56-issue-47-prc-idna-url-remediation/implementation_plan.md
  - [x] docs/pr/PR-56-issue-47-prc-idna-url-remediation/task.md

## 1) Snapshot (Issue)
- [x] `gh issue view 47 --json title,body,state,url` → pasted into SOT
- [x] `gh issue view 47 --comments --json comments | jq -r '.comments[].body' | sed -n '1,200p'` → pasted into SOT

## 2) Snapshot (Dependency BEFORE)
- [x] `cargo tree -i idna` → pasted into SOT
- [x] `cargo tree -i url`  → pasted into SOT
- [x] `cargo tree -p url`  → recorded in SOT
- [x] `cargo tree -p idna` → recorded in SOT

## 3) Remediation (choose one path)
- [x] Path-A: `idna` via `url` (already `url=2.5.4`, `idna=1.1.0`)
  - [x] No update required (already meets target)
- [ ] Path-B: `idna` direct

## 4) Snapshot (Dependency AFTER)
- [x] No change (already compliant). AFTER section filled in SOT as “same as BEFORE”.

## 5) Verify (Always Run Contract)
- [x] `cargo test --workspace` PASS
- [x] `nix run .#prverify` PASS
- [x] Save evidence: docs/evidence/prverify/prverify_20260211T120918Z_8018f93.md
- [x] `cockpit check` PASS

## 6) Doc finalize
- [x] SOT filled with snapshots + decision + evidence
- [ ] PR URL added to SOT Meta (after PR creation)

## 7) Commit + PR
- [x] `git diff --name-only` sanity (docs/pr + docs/evidence only)
- [x] `git status --porcelain=v1` clean except intended
- [ ] Commit message (docs-only):
  - [ ] `docs(pr56): prove idna/url already compliant (Issue #47 PR-C)`
- [ ] `git push -u origin feature/pr56-issue47-prc-idna-url-remediation`
- [ ] Open PR and link Issue #47
- [ ] After PR creation: fill PR URL in SOT Meta and check it
