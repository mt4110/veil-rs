# PR58 Task — Issue #47 Closeout (RunAlways)

> Labels:
> - RUN_ALWAYS: must run even if “already done”
> - IF / ELSE: branching control
> - FOR: loops/pagination
> - ERR: error handling
> - TEST: verification assertions

---

## 0) PATH_DISCOVERY (RUN_ALWAYS)

```bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
echo "ROOT=$ROOT"

# Canonical targets (preferred)
PLAN="$ROOT/docs/pr/PR-58-issue-47-closeout.plan.md"
TASK="$ROOT/docs/pr/PR-58-issue-47-closeout.task.md"
SOT ="$ROOT/docs/pr/PR-58-issue-47-closeout.md"

EV_DIR="$ROOT/.local/evidence/issue-47"
PRV_DIR="$ROOT/.local/prverify"
```

IF docs/pr does not exist:

```bash
# IF_FAIL: locate docs folder
ls -la "$ROOT/docs" || true
find "$ROOT" -maxdepth 3 -type d -name pr -o -name docs | sed -n '1,80p'
# ERR: stop and set correct PLAN/TASK/SOT paths before continuing.
```

## 1) BRANCH + CLEAN START (RUN_ALWAYS)

```bash
cd "$ROOT"
git switch main
git pull --ff-only

# TEST: working tree clean
test -z "$(git status --porcelain=v1)" && echo "[OK] clean" || (git status --porcelain=v1; exit 1)

git switch -c feature/pr58-issue47-closeout
```

## 2) DEPENDABOT OPEN=0 SNAPSHOT (RUN_ALWAYS)
### 2.1 Resolve repo slug (RUN_ALWAYS)

```bash
# TEST: gh available
command -v gh >/dev/null || (echo "ERR: gh not found"; exit 1)

# TEST: auth
gh auth status || (echo "ERR: gh not authenticated"; exit 1)

REPO="$(gh repo view --json nameWithOwner -q .nameWithOwner)"
echo "REPO=$REPO"
```

### 2.2 Snapshot “open alerts” (RUN_ALWAYS)

```bash
mkdir -p "$EV_DIR"

TS="$(date -u +%Y%m%dT%H%M%SZ)"
OUT="$EV_DIR/dependabot-open-alerts.$TS.json"
META="$EV_DIR/dependabot-open-alerts.$TS.meta.txt"

# FOR: pagination-safe (even though we expect 0)
# NOTE: --paginate emits multiple JSON arrays; jq -s 'add|length' merges them.
gh api --paginate -H "Accept: application/vnd.github+json" \
  "/repos/$REPO/dependabot/alerts?state=open&per_page=100" > "$OUT" \
  || (echo "ERR: dependabot api failed" | tee "$META"; exit 1)

COUNT="$(cat "$OUT" | jq -s 'add | length')"
{
  echo "timestamp_utc=$TS"
  echo "repo=$REPO"
  echo "endpoint=/repos/$REPO/dependabot/alerts?state=open&per_page=100 (--paginate)"
  echo "open_count=$COUNT"
} | tee "$META"

# TEST: expect 0
test "$COUNT" = "0" && echo "[OK] dependabot open alerts = 0" || (echo "ERR: open alerts != 0"; exit 1)
```

## 3) QUIET MAIN CONFIRMATION (RUN_ALWAYS)
### 3.1 Run prverify (RUN_ALWAYS)

```bash
# Keep evidence in repo
mkdir -p "$PRV_DIR"

# Run prverify on current branch (which is based on up-to-date main)
nix run .#prverify
```

### 3.2 Locate latest prverify report for current HEAD (RUN_ALWAYS)

```bash
HEAD7="$(git rev-parse --short=7 HEAD)"
LATEST="$(find "$PRV_DIR" -maxdepth 1 -type f -name "prverify_*_${HEAD7}.md" -print | sort -r | head -n 1)"

# TEST: must exist
test -n "${LATEST:-}" && test -f "$LATEST" && echo "[OK] prverify evidence: $LATEST" || (echo "ERR: prverify report not found for HEAD=$HEAD7"; ls -la "$PRV_DIR" | sed -n '1,120p'; exit 1)
```

IF you need stricter “quiet” assertion:

```bash
# Optional TEST: ensure no WARN in the prverify report (policy-dependent)
if rg -n "\bWARN\b" "$LATEST" >/dev/null; then
  echo "ERR: WARN found in prverify report: $LATEST"
  rg -n "\bWARN\b" "$LATEST" || true
  exit 1
else
  echo "[OK] no WARN in prverify report"
fi
```

## 4) WRITE DOCS (RUN_ALWAYS)
### 4.1 Write Plan/Task/SOT files (RUN_ALWAYS)

Create docs/pr dir if needed:

```bash
mkdir -p "$(dirname "$PLAN")"
```

Write PLAN:

```bash
cat > "$PLAN" <<'EOF'
<PASTE Plan.md CONTENT HERE>
EOF
```

Write TASK:

```bash
cat > "$TASK" <<'EOF'
<PASTE Task.md CONTENT HERE>
EOF
```

Write SOT (closeout note):

```bash
cat > "$SOT" <<EOF
# PR58 — Issue #47 Closeout (Evidence + Quiet Main)

## Closeout Summary

Issue #47 is resolved by **PR56** and **PR57**.
This PR (PR58) finalizes evidence pointers and confirms baseline “quiet main”.

## Evidence

- Dependabot open alerts snapshot: \`$OUT\`
  - Meta: \`$META\`
  - Expected: open_count=0
- Quiet main (prverify): \`$LATEST\`

## Links

- PR56: <LINK_HERE>
- PR57: <LINK_HERE>
- Issue #47: <LINK_HERE>

## Closing Note (for Issue #47)

Resolved by PR56 + PR57; checks green; Dependabot open alerts = 0; quiet main confirmed (see evidence pointers above). Closing.
EOF
```

### 4.2 Optional: runbook note (IF)
# IF: a runbook exists for dependabot/security
rg -n "Dependabot" "$ROOT/docs" || true


IF a suitable file exists (example):
- docs/runbook/security.md
- docs/runbook/dependabot.md

THEN append a tiny note:

```bash
# Example target (adjust if found)
RB="$ROOT/docs/runbook/dependabot.md"

mkdir -p "$(dirname "$RB")"
cat >> "$RB" <<'EOF'

## Dependabot lag note

Dependabot alerts can lag. Always snapshot **pre/post** state using the API and re-check that **open=0** before declaring the baseline clean.
EOF
```

## 5) ISSUE #47 COMMENT + CLOSE (RUN_ALWAYS)
### 5.1 Prepare comment body (RUN_ALWAYS)

```bash
ISSUE=47
BODY="$EV_DIR/issue47-closeout-comment.$TS.md"

cat > "$BODY" <<EOF
Resolved by PR56 + PR57.

- Checks: green
- Dependabot: open alerts = 0 (snapshot: \`$META\`)
- Quiet main: confirmed (prverify: \`$LATEST\`)

Closing Issue #47.
EOF
```

### 5.2 Comment + close (RUN_ALWAYS)

```bash
STATE="$(gh issue view "$ISSUE" --repo "$REPO" --json state -q .state)"
echo "Issue #$ISSUE state=$STATE"

# Comment always (even if already closed)
gh issue comment "$ISSUE" --repo "$REPO" --body-file "$BODY"

IF [ "$STATE" = "OPEN" ]; then
  gh issue close "$ISSUE" --repo "$REPO"
  echo "[OK] closed Issue #$ISSUE"
else
  echo "[OK] Issue #$ISSUE already closed"
fi
```

## 6) COMMIT + PR (RUN_ALWAYS)

```bash
cd "$ROOT"

# Stage only intended artifacts
git add "$PLAN" "$TASK" "$SOT" "$EV_DIR" "$PRV_DIR"

# TEST: show diff summary
git diff --cached --stat

git commit -m "docs(pr58): finalize Issue #47 closeout evidence"
git push -u origin feature/pr58-issue47-closeout
```

Create PR (example):

```bash
gh pr create --repo "$REPO" \
  --title "PR58: Issue #47 closeout (evidence + quiet main)" \
  --body "Final evidence pointers for closing Issue #47. Links PR56+PR57, pins dependabot open=0 snapshot + prverify quiet main evidence." \
  --base main
```

## 7) POST-MERGE SANITY (TEST)

After merge, on main:

```bash
git switch main
git pull --ff-only
nix run .#prverify
```

TEST:
- Issue #47 is closed
- open alerts still 0
- prverify green
