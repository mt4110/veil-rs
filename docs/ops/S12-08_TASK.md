# S12-08 TASK: S12-07 SOT closeout (STATUS update + PR92 doc fix)

> Rules: stopless / no-exit / no-heavy / split steps / log OBS

- Branch: s12-08-sot-closeout-pr92-v1
- PR fixed: #92 (merged, commit 33ab2bd)

---

## 0) Switch branch (light)
- [ ] Create branch

```bash
bash -lc '
cd "$(git rev-parse --show-toplevel 2>/dev/null || true)" 2>/dev/null || true
git switch -c s12-08-sot-closeout-pr92-v1 2>/dev/null || true
git status -sb 2>/dev/null || true
echo "OK: phase=end"
'
```

## 1) Discovery (light, stopless)

目的：SOT対象が実在することを “観測ログ” に残す（嘘をつかない）

- [ ] Capture STATUS S12-07 row and docs/pr listing

```bash
bash -lc '
ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [ -z "$ROOT" ]; then
  echo "ERROR: not_in_repo"
  echo "OK: phase=end stop=1"
else
  cd "$ROOT" 2>/dev/null || true

  TS="$(date -u +%Y%m%dT%H%M%SZ)"
  OBS=".local/obs/s12-08_kickoff_${TS}"
  mkdir -p "$OBS" 2>/dev/null || true
  echo "OK: obs_dir=$OBS"

  rg -n --no-heading -S "^\| S12-07 " docs/ops/STATUS.md 2>/dev/null | tee "$OBS/status_s12-07.txt" || true
  ls -la docs/pr 2>/dev/null | tee "$OBS/ls_docs_pr.txt" || true

  # already-existing PR-92 doc name collision check
  rg -n --no-heading -S "PR-92-s12-07-guard-sot-docnames-stdout-audit\.md" -S docs/pr 2>/dev/null | tee "$OBS/find_pr92_docname.txt" || true

  echo "OK: phase=end"
fi
'
```

## 2) Create S12-08 PLAN/TASK docs (light)

- [ ] Create `docs/ops/S12-08_PLAN.md`
- [ ] Create `docs/ops/S12-08_TASK.md`

## 3) Create PR-92 doc (light)

- [ ] Create `docs/pr/PR-92-s12-07-guard-sot-docnames-stdout-audit.md`

## 4) Patch STATUS.md (light, stopless, no-lies)

方針：可能なら自動パッチ。合わなければ ERROR を出して STOP=1（以降 SKIP）
※ここは “間違って更新” が最悪なので、合わないときは止める（exit ではなく STOP=1）

- [ ] Run compliant patcher (no sys.exit/SystemExit/assert)

```bash
bash -lc '
ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
STOP="0"

if [ -z "$ROOT" ]; then
  echo "ERROR: not_in_repo"
  STOP="1"
else
  cd "$ROOT" 2>/dev/null || true

  TS="$(date -u +%Y%m%dT%H%M%SZ)"
  OBS=".local/obs/s12-08_status_patch2_${TS}"
  mkdir -p "$OBS" 2>/dev/null || true
  echo "OK: obs_dir=$OBS"

  python3 - <<'"'"'PY'"'"' 2>&1 | tee "$OBS/status_patch2.log" || true
import re

STATUS_PATH = "docs/ops/STATUS.md"
PR92_DOC = "docs/pr/PR-92-s12-07-guard-sot-docnames-stdout-audit.md"
PLAN08 = "docs/ops/S12-08_PLAN.md"

def pick_evidence_col(cols):
  for i in range(2, len(cols)):
    c = cols[i].strip()
    if ("docs/" in c) or c.endswith(".md") or c.endswith(".md)"):
      return i
  return None

def parse_row(line):
  raw = line.strip("\n")
  segs = [s.strip() for s in raw.split("|")]
  if len(segs) < 4:
    return None
  if segs[0] != "" or segs[-1] != "":
    return None
  cols = segs[1:-1]
  return cols

def render_row(cols):
  return "| " + " | ".join(cols) + " |\n"

def patch_phase(src_lines, phase, new_progress, new_evidence, create_if_missing=False, insert_after_phase=None):
  found = False
  out = []
  inserted = False
  template_cols = None

  for line in src_lines:
    cols = parse_row(line)
    if cols and len(cols) >= 2 and cols[0] == phase:
      found = True
      template_cols = cols[:]  # capture shape
      cols[1] = new_progress
      ev_i = pick_evidence_col(cols)
      if ev_i is not None:
        cols[ev_i] = new_evidence
      out.append(render_row(cols))
      continue

    out.append(line)

    if (insert_after_phase is not None) and (not inserted):
      cols2 = parse_row(line)
      if cols2 and len(cols2) >= 1 and cols2[0] == insert_after_phase:
        template_cols = cols2[:]  # shape base
        inserted = True
        # insertion happens later if missing & allowed

  if (not found) and create_if_missing:
    if template_cols is None:
      return out, False, f"ERROR: cannot_create phase={phase} reason=no_template_row"
    new_cols = template_cols[:]
    new_cols[0] = phase
    if len(new_cols) >= 2:
      new_cols[1] = new_progress
    ev_i = pick_evidence_col(new_cols)
    if ev_i is not None:
      new_cols[ev_i] = new_evidence
    else:
      # no evidence-looking column, keep as-is but we still proceed (truthful log)
      pass

    # insert after insert_after_phase row if possible, else append at end
    inserted_out = []
    did_insert = False
    for line in out:
      inserted_out.append(line)
      cols = parse_row(line)
      if (not did_insert) and cols and cols[0] == (insert_after_phase or ""):
        inserted_out.append(render_row(new_cols))
        did_insert = True
    if not did_insert:
      inserted_out.append(render_row(new_cols))
    return inserted_out, True, None

  if not found and not create_if_missing:
    return out, False, f"ERROR: phase_row_missing phase={phase}"

  return out, True, None

def main():
  try:
    with open(STATUS_PATH, "r", encoding="utf-8") as f:
      src = f.readlines()
  except Exception as e:
    print(f"ERROR: read_failed path={STATUS_PATH} reason={e}")
    print("OK: patch_done stop=1")
    return

  # Patch S12-07 (must exist)
  out1, ok1, err1 = patch_phase(
    src, "S12-07",
    "100% (Merged PR #92)",
    PR92_DOC,
    create_if_missing=False
  )
  if not ok1:
    print(err1)
    print("OK: patch_done stop=1")
    return

  # Patch or create S12-08 (allowed create)
  out2, ok2, err2 = patch_phase(
    out1, "S12-08",
    "1% (WIP)",
    PLAN08,
    create_if_missing=True,
    insert_after_phase="S12-07"
  )
  if not ok2:
    print(err2)
    print("OK: patch_done stop=1")
    return

  try:
    with open(STATUS_PATH, "w", encoding="utf-8") as f:
      f.writelines(out2)
  except Exception as e:
    print(f"ERROR: write_failed path={STATUS_PATH} reason={e}")
    print("OK: patch_done stop=1")
    return

  print(f"OK: patched path={STATUS_PATH}")
  print("OK: patch_done stop=0")

main()
PY

  rg -n --no-heading -S "^\| S12-07 " docs/ops/STATUS.md 2>/dev/null | tee "$OBS/status_s12-07_after.txt" || true
  rg -n --no-heading -S "^\| S12-08 " docs/ops/STATUS.md 2>/dev/null | tee "$OBS/status_s12-08_after.txt" || true

fi

echo "OK: phase=end stop=$STOP"
'
```

## 5) Minimal verify (light)

- [ ] Check references are present (no heavy)

```bash
bash -lc '
ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [ -z "$ROOT" ]; then
  echo "ERROR: not_in_repo"
  echo "OK: phase=end stop=1"
else
  cd "$ROOT" 2>/dev/null || true
  TS="$(date -u +%Y%m%dT%H%M%SZ)"
  OBS=".local/obs/s12-08_verify_${TS}"
  mkdir -p "$OBS" 2>/dev/null || true
  echo "OK: obs_dir=$OBS"

  rg -n --no-heading -S "33ab2bd" docs/pr/PR-92-s12-07-guard-sot-docnames-stdout-audit.md 2>/dev/null | tee "$OBS/find_merge_commit.txt" || true
  rg -n --no-heading -S "prverify_20260224T073824Z_7362237\.md" docs/pr/PR-92-s12-07-guard-sot-docnames-stdout-audit.md 2>/dev/null | tee "$OBS/find_prverify.txt" || true
  rg -n --no-heading -S "veil-rs_review_strict_20260224_073756_736223723565\.tar\.gz" docs/pr/PR-92-s12-07-guard-sot-docnames-stdout-audit.md 2>/dev/null | tee "$OBS/find_strict_bundle.txt" || true

  echo "== git diff (names) =="
  git diff --name-only 2>/dev/null | tee "$OBS/git_diff_names.txt" || true

  echo "SKIP: heavy_verify reason=ci_is_evidence"
  echo "OK: phase=end"
fi
'
```

## 6) Commit / Push / PR (light)

- [ ] Commit docs only
- [ ] Push branch
- [ ] Create PR

```bash
bash -lc '
ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [ -z "$ROOT" ]; then
  echo "ERROR: not_in_repo"
  echo "OK: phase=end stop=1"
else
  cd "$ROOT" 2>/dev/null || true

  git add docs/ops/S12-08_PLAN.md docs/ops/S12-08_TASK.md docs/ops/STATUS.md docs/pr/PR-92-s12-07-guard-sot-docnames-stdout-audit.md 2>/dev/null || true
  git status -sb 2>/dev/null || true

  git commit -m "docs: close out S12-07 in SOT (PR-92) and start S12-08" 2>/dev/null || true
  git push -u origin s12-08-sot-closeout-pr92-v1 2>/dev/null || true

  # PR create (light)
  gh pr create --base main --head s12-08-sot-closeout-pr92-v1 \
    --title "S12-08: close out S12-07 in SOT (STATUS + PR-92 doc)" \
    --body "S12-07 merged (PR #92). This PR fixes SOT: add docs/pr PR-92 doc and update STATUS (S12-07=100% merged, S12-08=1% WIP). Evidence strings are pinned; heavy verify is SKIP (CI is evidence)." \
    2>/dev/null || true

  echo "OK: phase=end"
fi
'
```
