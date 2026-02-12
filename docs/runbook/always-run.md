# Always Run Runbook（PR儀式 / 証拠 / 禁則）

## Contract（PRごとに必須）
- SOT: `docs/pr/PR-<num>-<slug>.md`
- plan/task: `docs/pr/PR-<num>-<slug>/*`
- prverify evidence: `docs/evidence/prverify/prverify_<UTC>_<sha7>.md`
- `nix run .#prverify` PASS
- docs に `file:` と `//` の生連結を置かない

## Evidence（永続化）
- `.local` は消える。`docs/evidence` は永続。
- sha7 をキーに “該当shaの最新レポート” を拾ってコピーする。

## Forbidden（破綻しやすい行為）
- “とりあえずPR作る” → 禁止（契約不履行が溜まって破綻する）
- docs に `file:` + `//` の生連結 → 禁止
- 証拠を `.local` のまま放置 → 禁止

## Quick Recovery
- prverify FAIL → 直してから PASS の証拠だけを永続化
- docs 禁則 → `rg -n "file:/{2}" docs` で残存を洗い出し、分割表記に直す

## No-Heredoc / StrictMode-safe manual CLI patterns（手動運用の型）

目的：
- heredoc待ち（ターミナルが“入力待ちで固まる”）を根絶する
- `set -u` による「未初期化変数事故」を根絶する（特にプロンプト系 `__git_ps1` など）
- closeout / evidence を毎回同じ手順で回せるようにする（決定論）

### ルール（これだけ守れば事故らない）
- 手動実行のホスト側は `set -eo pipefail` を推奨（`-u` は付けない）
- `-u` が必要なら **必ず subshell に閉じ込める**（例：`bash -lc` の中）
- 重要変数は「毎回セットし直す」：`REPO` / `ISSUE` / `EVDIR`（ターミナル跨ぎ事故対策）
- 変数の未設定は `: "${VAR:?set VAR}"` で即死させる（嘘を付かない）
- ファイル生成は heredoc 禁止。**`printf` を使う**

### 最小セット（手動・安全）
```bash
set -eo pipefail
cd "$(git rev-parse --show-toplevel)"

REPO="mt4110/veil-rs"
ISSUE="59"
EVDIR="docs/evidence/pr59"

: "${REPO:?set REPO}" "${ISSUE:?set ISSUE}" "${EVDIR:?set EVDIR}"
mkdir -p "$EVDIR"

# set -u を使いたい時（subshell に封印）
env REPO="mt4110/veil-rs" ISSUE="59" EVDIR="docs/evidence/pr59" \
bash -lc 'set -euo pipefail
: "${REPO:?}" "${ISSUE:?}" "${EVDIR:?}"
cd "$(git rev-parse --show-toplevel)"
mkdir -p "$EVDIR"
'

# heredoc を使わないファイル生成（printf）
printf "%s\n" \
  "# Evidence note" \
  "" \
  "- repo: ${REPO}" \
  "- issue: ${ISSUE}" \
  > "${EVDIR}/note.md"

# prverify evidence の永続化（.local → docs/evidence）
set -eo pipefail
cd "$(git rev-parse --show-toplevel)"

HEAD7="$(git rev-parse --short=7 HEAD)"
LATEST="$(find .local/prverify -maxdepth 1 -type f -name "prverify_*_${HEAD7}.md" -print | sort -r | head -n 1)"
test -n "$LATEST" && test -f "$LATEST"

mkdir -p docs/evidence/prverify
cp -a "$LATEST" "docs/evidence/prverify/$(basename "$LATEST")"
```


