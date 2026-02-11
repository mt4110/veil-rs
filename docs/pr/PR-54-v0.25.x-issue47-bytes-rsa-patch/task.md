# PR54 Task — Issue #47 (bytes/rsa patch updates)

## 0) Setup

- [ ] `git status --porcelain` is clean
- [ ] on latest main: `git switch main && git pull --ff-only`
- [ ] create branch: `git switch -c feature/pr54-issue47-bytes-rsa-patch`

## 1) Create docs (Always Run Contract)

- [ ] Add SOT:
  - `docs/pr/PR-54-v0.25.x-issue47-bytes-rsa-patch.md`
- [ ] Add plan/task directory:
  - `docs/pr/PR-54-v0.25.x-issue47-bytes-rsa-patch/implementation_plan.md`
  - `docs/pr/PR-54-v0.25.x-issue47-bytes-rsa-patch/task.md`

## 2) Pre-check current versions (lockfile)

- [ ] Confirm current bytes version in Cargo.lock
- [ ] Confirm current rsa version in Cargo.lock

Suggested commands:
- `rg -n 'name = "bytes"|version = ' Cargo.lock`
- `rg -n 'name = "rsa"|version = ' Cargo.lock`

(Optional)
- [ ] `cargo tree -i bytes`
- [ ] `cargo tree -i rsa`

## 3) Apply patch updates (precise + minimal)

- [ ] `cargo update -p bytes --precise 1.11.1`
- [ ] `cargo update -p rsa --precise 0.9.10`

- [ ] Re-check Cargo.lock contains the intended versions

## 4) Verify

- [ ] `cargo test --workspace`
- [ ] `nix run .#prverify`

## 5) Archive evidence (docs/evidence)

- [ ] Identify latest `.local/prverify/prverify_*.md`
- [ ] Copy to:
  - `docs/evidence/prverify/prverify_<UTC>_<sha7>.md`

Recommended flow:
- [ ] `mkdir -p docs/evidence/prverify`
- [ ] `ls -1t .local/prverify/prverify_*.md | head -n 1`
- [ ] `git rev-parse --short=7 HEAD`
- [ ] copy/rename into docs/evidence/prverify

Guard:
- [ ] Ensure docs do not contain raw `file://`-style strings
  - If it fails, sanitize only the offending strings in the archived evidence file.

## 6) Update SOT pointer

- [ ] Update SOT line:
  - `Latest prverify report: docs/evidence/prverify/prverify_<UTC>_<sha7>.md`

## 7) Commit / PR

- [ ] `git diff` is minimal (prefer Cargo.lock + docs)
- [ ] Commit message suggestion:
  - `chore(pr54): bump bytes/rsa patch versions (Issue #47)`
- [ ] `git push -u origin feature/pr54-issue47-bytes-rsa-patch`
- [ ] Open PR targeting main, reference Issue #47

## 8) Merge

- [ ] Confirm CI green
- [ ] Merge (strategy per repo policy)
- [ ] Delete branch (remote/local) after merge
