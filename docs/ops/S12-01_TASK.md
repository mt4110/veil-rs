# S12-01 TASK — strict evidence binding: allow local prverify evidence

> Rule: heavy tasks are split. No fail-fast shell flags. One step = one observation.

## A) Preflight / cleanup (repo must be clean for strict)
- [ ] `cd "$(git rev-parse --show-toplevel)"`
- [ ] `git status -sb`
- [ ] If untracked exists under `docs/evidence/prverify/` (copied report), isolate it:
  - [ ] `mkdir -p .local/archive/prverify/20260218`
  - [ ] `ls -la docs/evidence/prverify | sed -n '1,40p'`
  - [ ] `cp -p docs/evidence/prverify/prverify_20260218T025648Z_756866a.md .local/archive/prverify/20260218/`
  - [ ] `rm -f docs/evidence/prverify/prverify_20260218T025648Z_756866a.md`
  - [ ] `git status -sb`

## B) Path capture (real file paths; paste outputs into PR)
- [ ] Locate evidence collection paths:
  - [ ] `rg -n "docs/evidence/prverify|review/evidence/prverify|E_EVIDENCE" -S .`
- [ ] Locate strict clean check:
  - [ ] `rg -n "repository is dirty|E_CONTRACT|git status|--porcelain" -S .`
- [ ] Locate where tar entries are written for evidence:
  - [ ] `rg -n "review/evidence|evidence/prverify|tar\\.|archive/tar" -S .`
- [ ] Record the exact file paths that must be edited (SOT: real paths only).

## C) Implement: include local prverify evidence in strict bundle
- [ ] Edit the file(s) identified in B) so that strict create also:
  - [ ] scans `.local/prverify/prverify_*.md` newest-first (limit N)
  - [ ] selects the newest file containing full HEAD SHA
  - [ ] includes it into tar at `review/evidence/prverify/<basename>`
- [ ] Implement atomic write:
  - [ ] write tar to `*.tmp`
  - [ ] self-audit against tmp
  - [ ] rename tmp → final on PASS
  - [ ] ensure failure does not leave a final-named tar behind

## D) Tests (small, local-only)
- [ ] Update/add unit tests to cover:
  - [ ] strict bundle passes when `.local/prverify` has a report containing HEAD SHA
  - [ ] strict bundle fails with a clear message when not found
  - [ ] (optional) dead-tar prevention: final file name not created on failure
- [ ] Run only the smallest Go test scope first:
  - [ ] `go test -count=1 ./cmd/reviewbundle -run Test -v`

## E) Local evidence generation (one step)
- [ ] `nix run .#prverify`
- [ ] Confirm the report exists:
  - [ ] `ls -lt .local/prverify | sed -n '1,30p'`
  - [ ] `git rev-parse HEAD`

## F) Strict bundle create + verify (two steps)
- [ ] Create:
  - [ ] `go run ./cmd/reviewbundle create --mode strict --out-dir .local/review-bundles`
- [ ] Verify:
  - [ ] `ls -lt .local/review-bundles | sed -n '1,30p'`
  - [ ] `BUNDLE_STRICT="$(ls -t .local/review-bundles/*_strict_*.tar.gz | head -n 1)"; echo "$BUNDLE_STRICT"`
  - [ ] `go run ./cmd/reviewbundle verify "$BUNDLE_STRICT"`

## G) Evidence inspection (cheap)
- [ ] Confirm the local prverify report is inside the tar:
  - [ ] `tar -tzf "$BUNDLE_STRICT" | rg "review/evidence/prverify/prverify_.*756866a\\.md"`
- [ ] Confirm self-audit / verify output says PASS.

## H) Docs / STATUS
- [ ] Update `docs/ops/STATUS.md` row for S12-01:
  - [ ] set progress to `99% (Review)` when PR open + CI pass
  - [ ] update Last Updated timestamp only
