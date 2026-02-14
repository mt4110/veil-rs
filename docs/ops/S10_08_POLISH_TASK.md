# S10-08 Final Polish Task

## 0. Preflight (Clean rail)
- [ ] cd "$(git rev-parse --show-toplevel)"
- [ ] git status -sb (Must be clean or "WIP commit" clean)

## 1. SOT Check
- [ ] List PR docs: `ls -la docs/pr | rg "s10-08|S10-08|prkit"`
- [ ] Check SOT naming convention (PR-XX)
  - [ ] Next number? `ls docs/pr` checking max number.
- [ ] Ensure SOT has:
  - [ ] Scope/Contract Summary
  - [ ] Evidence Pointer

## 2. Re-verify Fix A (Duplicate Error)
- [ ] `go test -count=1 ./cmd/prkit`
- [ ] Assert "stderr error token count == 1" is passing

## 3. Clean Evidence Generation (Phase 2)
- [ ] Ensure git status clean (stash or commit WIP)
- [ ] `nix run .#prverify`
- [ ] Copy report: `cp .local/prverify/prverify_*.md docs/evidence/prverify/`
- [ ] Update SOT evidence link
- [ ] Update Task evidence link

## 4. Final Commit
- [ ] `git add .`
- [ ] `git commit --amend` (or new commit)
- [ ] `git push`
