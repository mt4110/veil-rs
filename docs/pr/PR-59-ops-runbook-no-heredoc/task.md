# PR59 Task — Ops Runbook: No-Heredoc / StrictMode-safe manual CLI patterns

Legend:
- if / else if / else
- for / continue / break
- error: stop immediately
- skip: record reason and move on

Rule:
- No heredoc in examples (use printf)
- Host side: set -eo pipefail (NOT -u)
- -u is allowed only inside bash -lc subshell

---

## 00) Preflight (RunAlways)

- [ ] cd repo root (error if not a git repo)
```bash
set -eo pipefail
cd "$(git rev-parse --show-toplevel)" || (echo "error: not a git repo"; exit 1)
```

- [ ] ensure target files exist (error if missing)
```bash
test -f docs/runbook/always-run.md || (echo "error: missing docs/runbook/always-run.md"; exit 1)
mkdir -p docs/pr/PR-59-ops-runbook-no-heredoc
```

## 01) Docs edit (RunAlways)

- [ ] Edit docs/runbook/always-run.md and add the new section:
  - No heredoc examples
  - host: set -eo pipefail only
  - subshell: bash -lc with set -euo pipefail
  - REPO/ISSUE/EVDIR reset pattern
  - prverify evidence pin pattern (.local -> docs/evidence)

- [ ] Create SOT + plan/task:
  - `docs/pr/PR-59-ops-runbook-no-heredoc.md`
  - `docs/pr/PR-59-ops-runbook-no-heredoc/plan.md`
  - `docs/pr/PR-59-ops-runbook-no-heredoc/task.md`

## 02) Verification & Evidence (RunAlways)

- [ ] Run prverify (must PASS)
```bash
nix run .#prverify || (echo "error: prverify failed"; exit 1)
```

- [ ] Pin latest prverify report to committed evidence dir (error if not found or already exists)
```bash
set -eo pipefail
HEAD7="$(git rev-parse --short=7 HEAD)"
LATEST="$(find .local/prverify -maxdepth 1 -type f -name "prverify_*_${HEAD7}.md" -print | sort -r | head -n 1)"
test -n "$LATEST" && test -f "$LATEST" || (echo "error: prverify report not found for HEAD7=$HEAD7"; exit 1)

PIN="docs/evidence/prverify/$(basename "$LATEST")"
test -f "$PIN" || (echo "error: pinned prverify missing: $PIN"; exit 1)

perl -pi -e 's|^(\s*-\s*\*\*Latest prverify report:\*\*\s*)(?:`[^`]*`|.*)$|$1`'"$PIN"'`|' \
  docs/pr/PR-59-ops-runbook-no-heredoc.md

# verify replacement happened (must contain PIN)
rg -n "Latest prverify report:" docs/pr/PR-59-ops-runbook-no-heredoc.md | rg -F "$PIN" >/dev/null \
  || (echo "error: SOT pin failed (PIN not found in SOT)"; exit 1)

mkdir -p docs/evidence/prverify
cp -a "$LATEST" "$PIN"
test -f "$PIN" || (echo "error: pinned evidence missing: $PIN"; exit 1)
```

- [ ] Update SOT “Latest prverify report” to the committed path (deterministic)
```bash
set -eo pipefail
# $PIN and $LATEST should be available from previous step in the same session, 
# but for task robustness we re-calc if needed or assume sequential run.
HEAD7="$(git rev-parse --short=7 HEAD)"
LATEST="$(find .local/prverify -maxdepth 1 -type f -name "prverify_*_${HEAD7}.md" -print | sort -r | head -n 1)"
PIN="docs/evidence/prverify/$(basename "$LATEST")"

perl -pi -e 's|^(\s*-\s*\*\*Latest prverify report:\*\*\s*)(?:`[^`]*`|.*)$|$1`'"$PIN"'`|' \
  docs/pr/PR-59-ops-runbook-no-heredoc.md

# verify replacement happened (must contain PIN)
rg -n "Latest prverify report:" docs/pr/PR-59-ops-runbook-no-heredoc.md | rg -F "$PIN" >/dev/null \
  || (echo "error: SOT pin failed (PIN not found in SOT)"; exit 1)
```

## 03) Commit & Push (RunAlways)

- [ ] Show diff (stop if empty — do not open PR with zero changes)
```bash
git status --porcelain=v1
git diff --stat
```

- [ ] Commit docs only
```bash
git add docs/runbook/always-run.md docs/pr/PR-59-ops-runbook-no-heredoc.md docs/pr/PR-59-ops-runbook-no-heredoc/ docs/evidence/prverify/
git commit -m "docs(pr59): add no-heredoc + strictmode-safe manual CLI runbook patterns"
git push
```
