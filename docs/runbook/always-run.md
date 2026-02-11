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
