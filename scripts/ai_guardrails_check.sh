#!/usr/bin/env bash
set -euo pipefail

# scripts/ai_guardrails_check.sh
# Local + CI guardrails for docs/ai + repo hygiene.
#
# Usage:
#   scripts/ai_guardrails_check.sh                 # runs all checks
#   scripts/ai_guardrails_check.sh all
#   scripts/ai_guardrails_check.sh mac-ghost
#   scripts/ai_guardrails_check.sh version-sync
#   scripts/ai_guardrails_check.sh docs-ai-min
#   scripts/ai_guardrails_check.sh ai-pack-smoke
#   scripts/ai_guardrails_check.sh json-syntax

CMD="${1:-all}"

# Resolve repo root (works in CI + local). If not in a git repo, fall back to current directory.
if git rev-parse --show-toplevel >/dev/null 2>&1; then
  REPO_ROOT="$(git rev-parse --show-toplevel)"
else
  REPO_ROOT="$(pwd)"
fi
cd "${REPO_ROOT}"

note(){ printf "== %s ==\n" "$*" >&2; }
fail(){ printf "ERROR: %s\n" "$*" >&2; exit 1; }

mac_ghost() {
  note "Detect macOS ghosts (.DS_Store / AppleDouble)"
  if ! command -v git >/dev/null 2>&1; then
    note "git not found; skipping mac-ghost check"
    return 0
  fi
  if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    note "not a git work tree; skipping mac-ghost check"
    return 0
  fi
  local bad
  bad="$(git ls-files | grep -E '(^|/)\.DS_Store$|(^|/)\._' || true)"
  if [[ -n "${bad}" ]]; then
    echo "Found forbidden macOS files tracked in git:" >&2
    echo "${bad}" >&2
    return 1
  fi
}

version_sync() {
  note "Check workspace + crate versions are consistent"
  [[ -f Cargo.toml ]] || { note "No Cargo.toml; skipping"; return 0; }

  # Workspace version (best-effort)
  local ws_ver
  ws_ver="$(awk -F'\"' '
    BEGIN{in_ws=0}
    /^\[workspace\.package\]/{in_ws=1; next}
    in_ws && /^version *=/ {print $2; exit}
  ' Cargo.toml || true)"
  local versions=""
  shopt -s nullglob
  for f in crates/*/Cargo.toml; do
    ver="$(awk -F'"' '/^version *=/ {print $2; exit}' "$f")"
    versions+="${f#crates/}:${ver}"$'\n'
  done
  shopt -u nullglob

  [[ -n "${versions}" ]] || { note "No crates/*/Cargo.toml found; skipping"; return 0; }

  local uniq_versions
  uniq_versions="$(printf '%s' "${versions}" | awk -F: '{print $2}' | sort -u)"
  local count
  count="$(printf '%s\n' "${uniq_versions}" | wc -l | tr -d ' ')"
  echo "Detected versions:" >&2
  printf '%s' "${versions}" >&2
  echo "Unique versions:" >&2
  printf '%s' "${uniq_versions}" >&2

  if [[ "${count}" != "1" ]]; then
    fail "Version mismatch across crates. Align versions before merging."
  fi

  # If workspace version is present, ensure it matches crate version
  if [[ -n "${ws_ver}" ]]; then
    local crate_ver
    crate_ver="$(printf '%s' "${uniq_versions}" | head -n 1)"
    if [[ "${ws_ver}" != "${crate_ver}" ]]; then
      fail "Workspace version (${ws_ver}) does not match crate version (${crate_ver})."
    fi
  fi
}

docs_ai_min() {
  note "Minimal docs/ai rules"
  test -s docs/ai/WORKFLOW_RULES.md
  test -s docs/ai/PUBLISH_TEMPLATE.md
  test -s docs/ai/RELEASE_BODY_TEMPLATE.md
  test -s docs/ai/X_TEMPLATE.md

  local h1count
  h1count="$(awk '
    BEGIN{in_fence=0; c=0}
    /^```/ {in_fence=!in_fence; next}
    in_fence==0 && /^# / {c++}
    END{print c+0}
  ' docs/ai/WORKFLOW_RULES.md)"

  if [[ "${h1count}" -ne 1 ]]; then
    fail "WORKFLOW_RULES.md must have exactly one H1 (# title). Found: ${h1count}"
  fi

  local fences
  fences="$(grep -c '^```' docs/ai/WORKFLOW_RULES.md || true)"
  if (( fences % 2 != 0 )); then
    fail "WORKFLOW_RULES.md has unclosed code fences (odd number of code fence markers). Found: ${fences}"
  fi
}

ai_pack_smoke() {
  note "Ensure scripts/ai_pack.sh runs"
  if ! command -v git >/dev/null 2>&1 || ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    note "not a git work tree; skipping ai-pack-smoke check"
    return 0
  fi

  [[ -x scripts/ai_pack.sh ]] || fail "scripts/ai_pack.sh is not executable."
  [[ -x scripts/publish_draft.sh ]] || fail "scripts/publish_draft.sh is not executable."

  local out
  out="$(mktemp -t ai_pack.XXXXXX.txt 2>/dev/null || mktemp)"
  scripts/ai_pack.sh origin/main "${out}"
  test -s "${out}"
}

json_syntax() {
  note "JSON syntax (cspell.json)"
  if command -v python3 >/dev/null 2>&1; then
    python3 -m json.tool cspell.json >/dev/null
  else
    note "python3 not found; skipping json syntax check"
  fi
}

run_all() {
  mac_ghost
  version_sync
  docs_ai_min
  ai_pack_smoke
  json_syntax
  note "All guardrails passed"
}

case "${CMD}" in
  all) run_all ;;
  mac-ghost|mac_ghost) mac_ghost ;;
  version-sync|version_sync) version_sync ;;
  docs-ai-min|docs_ai_min) docs_ai_min ;;
  ai-pack-smoke|ai_pack_smoke) ai_pack_smoke ;;
  json-syntax|json_syntax) json_syntax ;;
  *)
    echo "Unknown command: ${CMD}" >&2
    echo "Valid: all | mac-ghost | version-sync | docs-ai-min | ai-pack-smoke | json-syntax" >&2
    exit 2
    ;;
esac
