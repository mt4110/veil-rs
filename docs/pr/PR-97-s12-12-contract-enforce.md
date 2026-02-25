# PR-97: S12-12 reviewbundle contract v1 enforce (parse+validate+cross-check)

## Facts (UI is truth)
- PR #96: merged (checks passed / branch deleted)
- S12-11: 100% (Merged PR #96)
- S12-12: kickoff in this PR

## Goal
`review/meta/contract.json` を contract v1 として検証し、
SSOT（contract + SHA256SUMS + seal）の輪を閉じる。

## Changes
- contract v1: parse/validate
- verify: contract validate + cross-check(contract↔SHA256SUMS)
- create: contract.json を struct ベースで安定生成
- errors: contract 系 reason/code 追加
- tests: light + reproducible

## Evidence (OBS)
- `tests`: `.local/obs/s12-12_impl_test_20260225T145947Z` (PASS)
- `strict verify`: `.local/obs/s12-12_impl_verify_strict_20260225T150114Z` (PASS)
- `prverify`: PASS

## DoD
- strict verify: stop=0
- contract invalid/mismatch: stop=1
- prverify: PASS

## Design Notes（境界条件の明文化）

### bound_to_head の意味（wip で矛盾しない定義）
`bound_to_head=true` は「head SHA に一致する evidence が **少なくとも1つ** 存在する」を意味する。
wip bundle は過去の prverify 結果を複数含むが、最新 head SHA に紐づく evidence が存在していれば契約を満たす。
これは S12-09 の「head binding は"全件必須"ではなく"存在確認"」の方針と整合。

### evidence 内の `/Users/` パスについて
`/Users/masakitakemura/...` 等の絶対パスは evidence ファイルの**テキスト内容**（prverify ログ）に含まれるもの。
`REVIEWBUNDLE_PACK_CONTRACT_v1.md` が禁じる「絶対パス」は **tar entry の name フィールド**の話であり、evidence のテキスト内容への制約ではない。
verify 側でも scanEvidenceContent は forbidden **URL pattern**（`file:` + `//` 形式）を狙い撃ちしており、`/Users/` は検出対象外（intentional）。
