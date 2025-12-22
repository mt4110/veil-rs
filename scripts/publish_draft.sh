#!/usr/bin/env bash
# scripts/publish_draft.sh
# DEPRECATED: This script is a wrapper around scripts/ai/gen.sh.

echo "WARN: scripts/publish_draft.sh is DEPRECATED." >&2
echo "WARN: Please use scripts/ai/gen.sh instead." >&2
echo "WARN: Forwarding call to scripts/ai/gen.sh..." >&2
echo ""

# Forward arguments to the new single entry point
# Map $1 (VERSION) and $2 (BASE_REF) to new flags
VERSION="${1:-}"
BASE_REF="${2:-origin/main}"

if [[ -z "${VERSION}" ]]; then
   # Let gen.sh handle the missing arg error, but pass empty to trigger usage
   exec scripts/ai/gen.sh
else
   exec scripts/ai/gen.sh --version "${VERSION}" --base-ref "${BASE_REF}"
fi
