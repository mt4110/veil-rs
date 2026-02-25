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
- (paste prverify + key logs here)

## DoD
- strict verify: stop=0
- contract invalid/mismatch: stop=1
- prverify: PASS
