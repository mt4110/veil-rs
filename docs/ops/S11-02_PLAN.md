# S11-02 PLAN — SOT guidance truth (stopless design)

## Goal
Eliminate stale guidance that says `veil sot new` (command does not exist).
Replace guidance with truthful, copy-pasteable manual SOT creation steps.
Behavior MUST NOT change — message-only edits.

## Non-Goals
- Do not change CI conditions, filenames checked, or failure logic.
- Do not refactor templates/docs beyond the minimum necessary to remove the stale command.

## Target Files (initial set; final set is discovered by rg)
- .github/workflows/pr_sot_guard.yml
- .github/PULL_REQUEST_TEMPLATE/*.md
- docs/pr/README.md
- docs/pr/sot_template.md
- docs/v0.20.0-planning/command_test_matrix.md
- (others found by rg)

## Canonical Truth (what to say instead)
Manual SOT creation:
1) Create a SOT file under `docs/pr/`:
   - `docs/pr/PR-<PR_NUMBER>-<slug>.md`
2) Fill it with the standard sections:
   - SOT / What / Verification / Evidence
3) Commit + push, then re-run the gate (CI/prverify).

## Pseudocode (if/else/for/continue/try/catch/error)
try:
  # 0) Preconditions
  if git status is dirty:
    continue (allowed) with 1-line reason recorded in commit message scope
  else:
    continue

  # 1) Discover: enumerate all stale guidance
  hits := rg -n "veil sot new|SOT Missing|Check SOT existence" in (.github, docs)
  if hits is empty:
    stop (success): nothing to change

  # 2) Decide replacement text (single source of truth)
  msg := Canonical Truth block (manual steps; copy-pasteable)
  rule := "message-only edits"

  # 3) Apply changes file-by-file
  for each file in hits:
    if file is ".github/workflows/pr_sot_guard.yml":
      edit ONLY echo/message strings
      continue
    else if file under ".github/PULL_REQUEST_TEMPLATE/":
      replace ONLY the stale command section with msg
      continue
    else if file under "docs/":
      replace ONLY the stale command section with msg
      continue
    else:
      continue (skip): out of scope; record 1-line skip reason in PR body

  # 4) Assert: zero stale command remains
  left := rg -n "veil sot new" in (.github, docs)
  if left is not empty:
    error: stop immediately; fix remaining references

  # 5) Gates
  run: nix run .#prverify
  expect PASS
  ensure prverify evidence is committed under docs/evidence/prverify/

catch:
  error: stop immediately with the exact failing command + file/line reference.
