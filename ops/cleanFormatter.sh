#!/usr/bin/env bash
set -euo pipefail

# Check only staged Markdown files (ACM = Added/Copied/Modified)
mapfile -t files < <(git diff --cached --name-only --diff-filter=ACM | grep -E '\.md$' || true)

if [ "${#files[@]}" -eq 0 ]; then
  exit 0
fi

bad=0

for f in "${files[@]}"; do
  [ -f "$f" ] || continue

  # Block raw file URLs. (doc-links guardと同じ思想)
  if grep -nE 'file://+' "$f" >/dev/null 2>&1; then
    echo "[FAIL] forbidden raw file URL detected in: $f"
    grep -nE 'file://+' "$f" || true
    echo ""
    bad=1
  fi
done

if [ "$bad" -ne 0 ]; then
  cat <<'EOF'
Fix guide:
- Replace raw file URLs with repo-relative paths, OR
- If you need to mention file URLs in docs, write them guard-safe (e.g. `file:` + `//` separated),
  so the raw substring `file://` does not appear.
EOF
  exit 1
fi

exit 0
