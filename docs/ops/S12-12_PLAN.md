# S12-12 PLAN: reviewbundle contract v1 enforce (parse+validate+cross-check)

## Goal
`review/meta/contract.json` を contract v1 として実装が検証し、
SSOT（contract + SHA256SUMS + seal）の輪を閉じる。

## Fixed Paths (bundle-root relative)
- review/meta/contract.json
- review/meta/SHA256SUMS
- review/meta/SHA256SUMS.sha256

## Non-Negotiable Rules
- stopless: 失敗は `ERROR: <reason> ... stop=1` で表現（終了コードで制御しない）
- heavy 禁止: テストは小さく、探索は分割、OBS に真実を残す

## Contract v1: Strategy (pseudo)
try:
  contract_bytes = read(contract.json)
  contract = parse_json(contract_bytes)
  validate_required_fields(contract)
  validate_schema_version(contract)
  validate_paths_are_safe(contract)  # 絶対パス/親走破など
  cross_check(contract, sha256sums_manifest)
catch:
  error contract_parse_or_validate_failed stop=1
finally:
  print final OK/ERROR + stop

## Cross-check Invariants (minimum)
- schema/version が v1 と一致
- mode (strict/wip) が整合
- contract が宣言する “bundle内ファイル集合” と SHA256SUMS が一致
- contract が宣言する固定パス 3点が正しい

## Deliverables
- cmd/reviewbundle/contract_v1.go (or similar): struct + parse + validate
- verify.go: contract validate を統合（stopless）
- create.go: contract を struct から生成（順序と安定性）
- errors.go: contract 系 reason/code を追加
- tests: contract parse/validate + contract↔sums mismatch

## DoD
- strict verify: stop=0
- contract invalid/mismatch: stop=1
- tests: light + reproducible
- prverify: PASS
