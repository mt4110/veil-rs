#!/usr/bin/env bash
set -euo pipefail

# Usage:
#   scripts/publish_draft.sh v0.12.1
# Behavior:
#   - writes draft files under ./dist/publish/
#   - does NOT push, tag, or call GitHub APIs (安全に「生成だけ」)

VERSION="${1:-}"
if [[ -z "${VERSION}" ]]; then
  echo "Usage: scripts/publish_draft.sh vX.Y.Z" >&2
  exit 2
fi

repo_root="$(git rev-parse --show-toplevel)"
cd "${repo_root}"

out_dir="dist/publish/${VERSION}"
mkdir -p "${out_dir}"

# 1) PR/Release/X drafts from templates
cp -f docs/ai/PUBLISH_TEMPLATE.md "${out_dir}/PUBLISH_${VERSION}.md"
cp -f docs/ai/RELEASE_BODY_TEMPLATE.md "${out_dir}/RELEASE_BODY_${VERSION}.md"
cp -f docs/ai/X_TEMPLATE.md "${out_dir}/X_${VERSION}.md"

# 2) ai_pack artifact for review/LLM input
pack_path="${out_dir}/AI_PACK_${VERSION}.txt"
scripts/ai_pack.sh origin/main "${pack_path}"

# 3) inject version placeholder (best-effort)
# mac sed needs -i '' ; GNU sed accepts -i
sed_inplace() {
    local f="$1"
    if sed --version >/dev/null 2>&1; then
        sed -i "s/vX.Y.Z/${VERSION}/g" "$f"
    else
        sed -i '' "s/vX.Y.Z/${VERSION}/g" "$f"
    fi
}

sed_inplace "${out_dir}/PUBLISH_${VERSION}.md"
sed_inplace "${out_dir}/RELEASE_BODY_${VERSION}.md"
sed_inplace "${out_dir}/X_${VERSION}.md"

echo "Generated:"
echo "  ${out_dir}/PUBLISH_${VERSION}.md"
echo "  ${out_dir}/RELEASE_BODY_${VERSION}.md"
echo "  ${out_dir}/X_${VERSION}.md"
echo "  ${pack_path}"
