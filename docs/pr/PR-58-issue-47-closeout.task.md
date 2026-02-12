# PR58 Task — Issue #47 Closeout (structured control words / ultra fine-grained)

Legend:
- if / else if / else
- for / continue / break
- error: stop immediately
- skip: record reason and move on
- TEST: assert; fail => error

Rule (degrade-model safe):
- Do NOT discuss “IsArtifact”. Do the work.
- Prefer overwrite (cat <<'EOF') over patch edits.
- If a step yields no change (no diff), stop and proceed (do not loop).
- Evidence pointers in docs must NEVER point to `.local/`. Use `docs/pr/evidence/issue-47/`.

---

## 00) Path Discovery (RunAlways)

- [ ] TEST: repo root exists; cd repo root
```bash
set -euo pipefail
ROOT="$(git rev-parse --show-toplevel)" || (echo "error: not a git repo"; exit 1)
cd "$ROOT"
echo "ROOT=$ROOT"


 if docs/pr missing -> create; else continue

if [ ! -d "docs/pr" ]; then
  mkdir -p "docs/pr"
fi


 set canonical paths (TEST)

PLAN="docs/pr/PR-58-issue-47-closeout.plan.md"
TASK="docs/pr/PR-58-issue-47-closeout.task.md"
SOT="docs/pr/PR-58-issue-47-closeout.md"
EVDIR="docs/pr/evidence/issue-47"

mkdir -p "$EVDIR"
test -d "docs" && test -d "docs/pr" && test -d "$EVDIR"

01) Branch Selection (RunAlways)

 TEST: working tree clean; else error

test -z "$(git status --porcelain=v1)" || (echo "error: dirty working tree"; git status --porcelain=v1; exit 1)


 for candidate branches -> if found break; else continue; after loop error

FOUND=""
for b in \
  "feature/pr58-issue47-closeout" \
  "feature/pr58-issue-47-closeout" \
  "pr58" \
  "feature/pr58"
do
  if git show-ref --verify --quiet "refs/heads/$b"; then
    FOUND="$b"
    break
  else
    continue
  fi
done

test -n "$FOUND" || (echo "error: PR58 branch not found"; git branch --all | sed -n '1,200p'; exit 1)

echo "BRANCH=$FOUND"
git switch "$FOUND"
git pull --ff-only

02) Tooling Preflight (RunAlways)

 TEST: gh exists

command -v gh >/dev/null || (echo "error: gh not installed"; exit 1)


 TEST: gh auth ok

gh auth status || (echo "error: gh not authenticated"; exit 1)


 set REPO slug (TEST)

REPO="$(gh repo view --json nameWithOwner -q .nameWithOwner)" || (echo "error: cannot read repo"; exit 1)
test -n "$REPO"
echo "REPO=$REPO"


 set timestamp

TS="$(date -u +%Y%m%dT%H%M%SZ)"
echo "TS=$TS"

03) Evidence: Dependabot open=0 Snapshot (RunAlways)

 capture JSON snapshot (error if empty)

DB_OUT="$EVDIR/dependabot-open-alerts.$TS.json"

gh api -H "Accept: application/vnd.github+json" \
  "/repos/$REPO/dependabot/alerts?state=open&per_page=100" > "$DB_OUT" \
  || (echo "error: dependabot api failed"; exit 1)

test -s "$DB_OUT" || (echo "error: empty dependabot snapshot"; exit 1)


 compute open_count (no jq) (TEST)

COUNT="$(python3 - <<PY
import json
with open("$DB_OUT","r",encoding="utf-8") as f:
    print(len(json.load(f)))
PY
)"
echo "open_count=$COUNT"


 write meta

DB_META="$EVDIR/dependabot-open-alerts.$TS.meta.txt"
{
  echo "timestamp_utc=$TS"
  echo "repo=$REPO"
  echo "endpoint=/repos/$REPO/dependabot/alerts?state=open&per_page=100"
  echo "open_count=$COUNT"
} | tee "$DB_META"


 TEST: open_count == 0 else error stop (do NOT claim clean)

test "$COUNT" = "0" || (echo "error: dependabot open alerts != 0"; exit 1)

04) Evidence: prverify Quiet Main (RunAlways)

 run prverify (error stop on failure)

nix run .#prverify || (echo "error: prverify failed"; exit 1)


 locate latest prverify report for current HEAD7 (TEST)

HEAD7="$(git rev-parse --short=7 HEAD)"
PRV_SRC=".local/prverify"

LATEST="$(find "$PRV_SRC" -maxdepth 1 -type f -name "prverify_*_${HEAD7}.md" -print | sort -r | head -n 1)"
test -n "${LATEST:-}" && test -f "$LATEST" || (echo "error: prverify report not found for HEAD7=$HEAD7"; ls -la "$PRV_SRC" | sed -n '1,200p'; exit 1)

echo "LATEST=$LATEST"


 copy to committed evidence dir (TEST)

cp -a "$LATEST" "$EVDIR/"
PRV_PIN="$EVDIR/$(basename "$LATEST")"
test -f "$PRV_PIN" || (echo "error: pinned prverify missing"; exit 1)
echo "PRV_PIN=$PRV_PIN"


 optional TEST: no WARN in report (enable if policy requires)

if rg -n "\bWARN\b" "$PRV_PIN" >/dev/null; then
  echo "error: WARN found in pinned prverify"
  rg -n "\bWARN\b" "$PRV_PIN" || true
  exit 1
else
  echo "skip: no WARN found"
fi

05) Issue #47 Closeout (RunAlways)

 write closeout comment body (committed copy)

ISSUE=47
ISSUE_COMMENT="$EVDIR/issue47-closeout-comment.$TS.md"

cat > "$ISSUE_COMMENT" <<EOF2
Resolved by PR56 + PR57.

- Checks: green
- Dependabot: open alerts = 0 (meta: \`$DB_META\`)
- Quiet main: confirmed (prverify: \`$PRV_PIN\`)

Closing Issue #47.
EOF2

test -s "$ISSUE_COMMENT" || (echo "error: issue comment file empty"; exit 1)
echo "ISSUE_COMMENT=$ISSUE_COMMENT"


 get issue state (TEST)

STATE="$(gh issue view "$ISSUE" --repo "$REPO" --json state -q .state)" || (echo "error: cannot view issue"; exit 1)
test -n "$STATE"
echo "Issue#$ISSUE state=$STATE"


 comment always

gh issue comment "$ISSUE" --repo "$REPO" --body-file "$ISSUE_COMMENT" \
  || (echo "error: failed to comment issue"; exit 1)


 if OPEN -> close; else if CLOSED -> skip close; else error

if [ "$STATE" = "OPEN" ]; then
  gh issue close "$ISSUE" --repo "$REPO" || (echo "error: failed to close issue"; exit 1)
elif [ "$STATE" = "CLOSED" ]; then
  echo "skip: already closed"
else
  echo "error: unexpected issue state=$STATE"
  exit 1
fi


 TEST: issue is CLOSED

gh issue view "$ISSUE" --repo "$REPO" --json state -q .state | rg -q "CLOSED" \
  || (echo "error: issue not CLOSED"; exit 1)

06) Pin SOT placeholders to exact evidence filenames (RunAlways)

 TEST: SOT exists

test -f "$SOT" || (echo "error: SOT missing: $SOT"; exit 1)


 replace placeholders {{DB_OUT}} {{DB_META}} {{PRV_PIN}} {{ISSUE_COMMENT}} (TEST)

python3 - <<PY
from pathlib import Path
p=Path("$SOT")
s=p.read_text(encoding="utf-8")

repl = {
  "{{DB_OUT}}": "$DB_OUT",
  "{{DB_META}}": "$DB_META",
  "{{PRV_PIN}}": "$PRV_PIN",
  "{{ISSUE_COMMENT}}": "$ISSUE_COMMENT",
}

for k,v in repl.items():
  s = s.replace(k, v)

# TEST: no placeholders remain
for k in repl.keys():
  if k in s:
    raise SystemExit(f"error: placeholder still present: {k}")

p.write_text(s, encoding="utf-8")
print("pinned:", p)
PY


 TEST: SOT has no .local/ pointers

if rg -n "\.local/" "$SOT" >/dev/null; then
  echo "error: SOT still contains .local pointers"
  rg -n "\.local/" "$SOT" || true
  exit 1
else
  echo "OK: SOT has no .local pointers"
fi

07) Commit / Push (RunAlways)

 stage only docs paths (TEST)

git add "$PLAN" "$TASK" "$SOT" "$EVDIR"
git diff --cached --stat


 TEST: no .local staged (error)

if git diff --cached --name-only | rg -n "^\.(local|local/)" >/dev/null; then
  echo "error: .local staged"
  git diff --cached --name-only | rg -n "^\.(local|local/)" || true
  exit 1
else
  echo "OK: no .local staged"
fi


 commit (if nothing staged -> skip)

if git diff --cached --quiet; then
  echo "skip: nothing to commit"
else
  git commit -m "docs(pr58): make Issue #47 closeout evidence pointerable"
fi


 push

git push

08) Final Verification (TEST)

 TEST: PR58 exists

gh pr view 58 --repo "$REPO" --json url,headRefName,state -q '.' || (echo "error: cannot view PR58"; exit 1)


 TEST: Issue #47 CLOSED

gh issue view 47 --repo "$REPO" --json state -q .state || (echo "error: cannot view issue 47"; exit 1)


 TEST: evidence files exist in repo tree

ls -la "$EVDIR" | sed -n '1,200p'


EOF
