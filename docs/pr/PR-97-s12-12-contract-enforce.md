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
