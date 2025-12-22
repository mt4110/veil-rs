#!/usr/bin/env bash
set -euo pipefail

# Usage:
#   scripts/ai_pack.sh [BASE_REF] [OUT]
# Example:
#   scripts/ai_pack.sh origin/main /tmp/ai_pack.txt
#
# Outputs:
#   - git status / summary
#   - changed files
#   - unified diff (with enough context)
#   - CONTEXT_MAP (key files + line-number excerpts)

BASE_REF="${1:-origin/main}"
OUT="${2:-}"

# Robustness: verify BASE_REF exists, fallback to HEAD~1, then HEAD
if ! git rev-parse --verify "${BASE_REF}" >/dev/null 2>&1; then
    echo "Note: BASE_REF '${BASE_REF}' not found." >&2
    if git rev-parse --verify "HEAD~1" >/dev/null 2>&1; then
        echo "Falling back to HEAD~1" >&2
        BASE_REF="HEAD~1"
    else
        echo "Falling back to HEAD (empty diff expected)" >&2
        BASE_REF="HEAD"
    fi
fi

tmp_out=""
if [[ -z "${OUT}" ]]; then
    tmp_out="$(mktemp -t ai_pack.XXXXXX.txt 2>/dev/null || mktemp)"
    OUT="${tmp_out}"
fi

repo_root="$(git rev-parse --show-toplevel)"
cd "${repo_root}"

now_utc="$(date -u +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null || true)"

write() { printf "%s\n" "$*" >> "${OUT}"; }

write "=== AI_PACK ==="
write "generated_at_utc: ${now_utc}"
write "base_ref: ${BASE_REF}"
write "head: $(git rev-parse --short HEAD)"
write

write "=== STATUS ==="
git status -sb >> "${OUT}" || true
write

write "=== SUMMARY ==="
write "branch: $(git branch --show-current 2>/dev/null || true)"
write "last_commits:"
git log --oneline -10 >> "${OUT}" || true
write

write "=== CHANGED FILES (BASE..HEAD) ==="
git diff --name-status "${BASE_REF}...HEAD" >> "${OUT}" || true
write

write "=== DIFF (unified=6) ==="
git diff --unified=6 "${BASE_REF}...HEAD" >> "${OUT}" || true
write

write "=== CONTEXT_MAP ==="
write "# Purpose: give line-numbered context for the important files"
write "# Format: FILE + nl excerpts (top + around changes)"
write

changed_files="$(git diff --name-only "${BASE_REF}...HEAD" || true)"

# Helper: safe nl (works on mac/linux)
nl_file() {
    local f="$1"
    # Skip binary or missing
    [[ -f "$f" ]] || return 0
    if command -v file >/dev/null 2>&1; then
        if file "$f" | grep -qiE 'image|audio|video|archive|compressed|binary'; then
            return 0
        fi
    fi
    nl -ba "$f"
}

# Decide "relevant" files: docs/ai, scripts, workflow, Cargo*, README, crates/*
relevant=()
while IFS= read -r f; do
    [[ -n "$f" ]] || continue
    case "$f" in
    docs/ai/*|scripts/*|.github/workflows/*|Cargo.toml|Cargo.lock|README.md|crates/*)
        relevant+=("$f")
        ;;
    *)
        # still include if small & texty
        relevant+=("$f")
        ;;
esac
done <<< "${changed_files}"

# De-dupe
uniq_relevant=()
for f in "${relevant[@]}"; do
    skip=""
    for u in "${uniq_relevant[@]}"; do
        [[ "$u" == "$f" ]] && skip="1" && break
    done
    [[ -z "$skip" ]] && uniq_relevant+=("$f")
done

for f in "${uniq_relevant[@]}"; do
    write "---- FILE: ${f} ----"
    # Show top 60 lines for orientation
    write "[head: top]"
    nl_file "$f" | sed -n '1,60p' >> "${OUT}" || true
    write

    # Show lines around diff hunks (best-effort)
    # Extract approximate line numbers from diff: +<start>,<len>
    hunk_starts="$(git diff -U0 "${BASE_REF}...HEAD" -- "$f" \
    | sed -n 's/^@@ .*+\([0-9][0-9]*\)\(,\([0-9][0-9]*\)\)\? @@.*/\1 \3/p' \
    | head -n 8 || true)"

    if [[ -n "${hunk_starts}" ]]; then
        write "[around changes]"
        while read -r start len; do
            [[ -n "${start}" ]] || continue
            # default window
            from=$(( start - 20 ))
            to=$(( start + 60 ))
            (( from < 1 )) && from=1
            nl_file "$f" | sed -n "${from},${to}p" >> "${OUT}" || true
            write
        done <<< "${hunk_starts}"
    fi

    write
done

write "=== END ==="

if [[ -n "${tmp_out}" ]]; then
  echo "${OUT}"
fi