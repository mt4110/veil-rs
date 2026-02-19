# S12-02 TASK — Closeout + Ritual Spec (zsh-safe)

## 0) Preflight (no heavy)
- [ ] `cd "$(git rev-parse --show-toplevel 2>/dev/null)" 2>/dev/null || true`
- [ ] `git fetch origin --prune 2>/dev/null || true`
- [ ] `git switch main 2>/dev/null || true`
- [ ] `git pull --ff-only 2>/dev/null || true`
- [ ] Observe:
  - `git status -sb 2>/dev/null || true`
  - `git log --oneline -n 12 2>/dev/null || true`

## 1) Branch (deterministic)
- [ ] `git switch -c s12-02-closeout-ritual-v1 2>/dev/null || true`

## 2) Create docs (if missing)
- [ ] Ensure files exist:
  - `ls -la docs/ops/S12-02_PLAN.md docs/ops/S12-02_TASK.md 2>/dev/null || true`
- [ ] If missing, copy templates (best-effort; no exit):
  - `test -f docs/ops/meta/DETERMINISTIC_PLAN_TEMPLATE.md && cp docs/ops/meta/DETERMINISTIC_PLAN_TEMPLATE.md docs/ops/S12-02_PLAN.md || true`
  - `test -f docs/ops/meta/DETERMINISTIC_TASK_TEMPLATE.md && cp docs/ops/meta/DETERMINISTIC_TASK_TEMPLATE.md docs/ops/S12-02_TASK.md || true`

## 3) Patch STATUS truth (S12-01 must be 100% now)
- [ ] Edit: `docs/ops/STATUS.md`
  - S12-01: `99% (Review)` -> `100% (Merged)`, Current -> `-`
  - S12-02: start as `1% (WIP)` with Current set to this phase one-liner
  - Update "Last Updated" block (Date/By/Agent/Evidence)
- [ ] Deterministic patch helper (no exit / no assert):
  - Run and inspect outputs:
    - `python3 - <<'PY'
from pathlib import Path

p = Path("docs/ops/STATUS.md")
want_evidence = "docs/evidence/ops/obs_20260218_s12-02.md"

try:
    txt = p.read_text(encoding="utf-8")
except Exception as e:
    print(f"ERROR: read failed: {e}")
    txt = ""

if not txt:
    print("ERROR: empty STATUS.md")
else:
    lines = txt.splitlines(True)
    changed = [False]

    def patch_row(prefix, new_progress=None, new_current=None):
        for i, ln in enumerate(lines):
            if ln.startswith(prefix):
                parts = ln.split("|")
                if len(parts) >= 5:
                    if new_progress is not None:
                        parts[3] = f" {new_progress} "
                    if new_current is not None:
                        parts[4] = f" {new_current} "
                    lines[i] = "|".join(parts)
                    changed[0] = True
                    print(f"OK: patched {prefix.strip()} line {i+1}")
                else:
                    print(f"ERROR: unexpected table format for {prefix.strip()}")
                return
        print(f"ERROR: row not found: {prefix.strip()}")

    patch_row("| S12-01 ", new_progress="100% (Merged)", new_current="-")
    patch_row("| S12-02 ", new_progress="1% (WIP)", new_current="S12-02: Closeout + ritual spec (zsh-safe)")

    # sync Last Updated Evidence
    hit = False
    for i, ln in enumerate(lines):
        if ln.startswith("- Evidence: "):
            hit = True
            new = f"- Evidence: {want_evidence}\n"
            if ln != new:
                lines[i] = new
                changed[0] = True
                print("OK: updated Last Updated Evidence")
            else:
                print("SKIP: Last Updated Evidence already synced")
            break
    if not hit:
        print("ERROR: Evidence line not found under Last Updated")

    if changed[0]:
        try:
            p.write_text("".join(lines), encoding="utf-8")
            print("OK: wrote docs/ops/STATUS.md")
        except Exception as e:
            print(f"ERROR: write failed: {e}")
    else:
        print("SKIP: no changes applied")
PY`

## 4) Ritual spec: zsh-safe observation (NO glob)
- [ ] In `docs/ops/S12-02_TASK.md` (this file), define the canonical snippet:

  - Replace any `ls review_bundle_*.tar.gz` with:
    - `find . -maxdepth 1 -type f -name 'review_bundle_*.tar.gz' -print 2>/dev/null || true`
    - `find . -maxdepth 1 -type f -name 'review_pack_*.tar.gz' -print 2>/dev/null || true`

  - Reason line (1 line):
    - `NOTE: zsh nomatch makes bare globs fail before command execution; find avoids this deterministically.`

## 5) Optional: light evidence file (no heavy)
- [ ] Create: `docs/evidence/ops/obs_20260218_s12-02.md`
  - Paste:
    - `git status -sb`
    - `git log --oneline -n 12`
    - `rg -n "^\| S12-01 |^\| S12-02 " docs/ops/STATUS.md`
    - `rg -n "^Last Updated:" -n docs/ops/STATUS.md`

## Ritual (Canonical): Strict Ritual Capsule
- **Goal**: Create a strict review bundle bound to HEAD, ensuring evidence exists.
- **Command**:
  - `go run ./cmd/reviewbundle create --mode strict --heavy=auto --autocommit --message "docs(ops): strict ritual" --out-dir .local/review-bundles`
- **Behavior**:
  - **Auto-Commit**: If dirty, commits with provided message (requires staged changes or will fail safe).
  - **Evidence**: Automatically finds newest `prverify_*.md` containing HEAD SHA.
  - **Heavy**: If evidence missing, runs `prverify` (heavy) automatically, then retries.
  - **Output**: Returns `OK`, `ERROR`, or `SKIP`. Process does not exit with code 1 unless compilation fails, allowing "stopless" chaining.
- **Zsh-Safe Observation**:
  - `find .local/review-bundles -maxdepth 1 -type f -name 'veil-rs_review_strict_*.tar.gz' -print | sort | tail -n 1`

## 6) Gates (keep light; no heavy required)
- [ ] `rg -n "S12-02" docs/ops/S12-02_PLAN.md docs/ops/S12-02_TASK.md 2>/dev/null || true`
- [ ] `rg -n "S12-01" docs/ops/STATUS.md 2>/dev/null || true`

## 7) Commit + PR skeleton
- [ ] Commit message:
  - `docs(ops): closeout S12-01 + define S12-02 ritual (zsh-safe)`
- [ ] PR body pointers:
  - SOT: `docs/ops/S12-02_PLAN.md`, `docs/ops/S12-02_TASK.md`
  - Evidence: `docs/evidence/ops/obs_20260218_s12-02.md` (if created)
