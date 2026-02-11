# v0.21.x〜v0.25.x — TASKS（Always Run）

## 1) PR共通 Always Run（毎PR必須）
### 1.1 Start（repo / clean）
```bash
cd "$(git rev-parse --show-toplevel)"
git status --porcelain=v1

1.2 Branch
git switch main
git pull --ff-only
git switch -c feature/pr<NUM>-<slug>

1.3 SOT / plan / task を最初に置く（空でも良い：存在が契約）

docs/pr/PR-<NUM>-<slug>.md

docs/pr/PR-<NUM>-<slug>/implementation_plan.md

docs/pr/PR-<NUM>-<slug>/task.md

1.4 Verify
nix run .#prverify

1.5 Evidence 永続化（docs/evidence へ）
SHA7="$(git rev-parse --short=7 HEAD)"
UTC="$(date -u +%Y%m%dT%H%M%SZ)"
mkdir -p docs/evidence/prverify

# .local から “該当sha7の最新” を探してコピー（場所が変わっても耐える）
SRC="$(find .local -maxdepth 3 -type f -name "prverify_*_${SHA7}.md" 2>/dev/null | sort -r | head -n 1)"
test -n "$SRC"
cp -a "$SRC" "docs/evidence/prverify/prverify_${UTC}_${SHA7}.md"

1.6 SOT へ反映（必須）

Latest prverify report: docs/evidence/prverify/prverify_<UTC>_<sha7>.md

1.7 Doc-links 規約（禁則）

docs に file: と // の生連結（= file: + //）を置かない

検索は “生連結を避けた表記” を使う（例：正規表現）

rg -n "file:/{2}" docs || true

1.8 Commit / Push
git add docs/pr docs/evidence/prverify docs/roadmap docs/templates docs/runbook
git commit -m "docs: lock in always-run ritual templates and roadmap"
git push -u origin feature/pr<NUM>-<slug>
