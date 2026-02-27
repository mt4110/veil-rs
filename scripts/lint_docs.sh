#!/usr/bin/env bash
set -euo pipefail

echo "Linting Documentation for B2B Readiness..."

# 1. VEIL_ORG_RULES is forbidden outside of Deprecated note
echo "Checking deprecated env var usage (VEIL_ORG_RULES)..."
if rg -n 'VEIL_ORG_RULES' README*.md 2>/dev/null | rg -v 'Deprecated|非推奨'; then
  echo "[FAIL] VEIL_ORG_RULES is used outside a 'Deprecated' or '非推奨' note. This risks B2B contract violations."
  exit 1
fi

# 2. README must recommend v1.0.0 and NEVER v0.17.0
echo "Checking install tag versions..."
if rg -n 'v0\.17\.0' README*.md 2>/dev/null; then
  echo "[FAIL] Found legacy v0.17.0 tags in README files. Must use v1.0.0."
  exit 1
fi
for doc in README.md README_EN.md; do
  if ! rg -q 'tag v1\.0\.0' "$doc"; then
    echo "[FAIL] $doc is missing the 'v1.0.0' install tag instruction."
    exit 1
  fi
done

# 3. Troubleshooting ignore must strictly use [core] ignore
echo "Checking gitconfig ignore structure..."
if rg -n 'ignore = \[' README.md 2>/dev/null | rg -v '\[core\]'; then
  echo "[FAIL] README.md has an unsafe 'ignore' structure. Must explicitly show '[core] ignore'."
  exit 1
fi

# 4. Strict CSP Enforcement for Frontend UI (checking compiled Svelte SPA)
echo "Checking Strict CSP compliance in Svelte dist/ ..."
if rg -n 'onclick=|style=|<style>|<script>' crates/veil-pro/frontend/dist/index.html 2>/dev/null; then
  echo "[FAIL] Found inline styles/scripts or bare style/onclick attributes in dist/index.html. Strict CSP violated!"
  exit 1
fi

if rg -n 'eval\(' crates/veil-pro/frontend/dist/assets/ 2>/dev/null; then
  echo "[FAIL] Found 'eval(' in frontend JS assets. Strict CSP violated!"
  exit 1
fi

# 5. Deprecated CLI flags prohibition (--fail-score)
echo "Checking for deprecated flags (--fail-score)..."
if rg -n -- '--fail-score' README*.md 2>/dev/null; then
  echo "[FAIL] Found deprecated '--fail-score' flag in READMEs. Must use '--fail-on-score' exclusively."
  exit 1
fi

echo "[PASS] Documentation B2B readiness contract verified."
exit 0
