# PR-89 — S12-06B: pack contract を"法律"にする

## SOT

- Scope: S12-06B — review bundle contract law (contract doc + verify CLI stopless + compat tests)
- Branch: s12-06-pack-contract-law-v1
- **Auto-merge disabled — do NOT enable auto-merge.**
- Deliverables:
  - docs/ops/REVIEW_BUNDLE_CONTRACT_v1.md (new: canonical contract reference)
  - cmd/reviewbundle/main.go (verify CLI → stopless OK: phase=end stop=)
  - cmd/reviewbundle/contract_law_test.go (new: 3 tests — StoplessOutput / V11Compat / SingleSource)
  - docs/ops/S12-06_PLAN.md (new)
  - docs/ops/S12-06_TASK.md (new)
  - docs/ops/STATUS.md (S12-06 row added, 99% Review)

## What

S12-06B は review bundle の contract を"法律"として明文化し、破壊的変更をテストで止める。

| DoD                                           | 実装                                                  |
| --------------------------------------------- | ----------------------------------------------------- |
| Contract v1 document                          | docs/ops/REVIEW_BUNDLE_CONTRACT_v1.md                 |
| contract_version = manifest のみ (唯一の真実) | 既存 contract.json を単一ソースとして文書化           |
| verify CLI が stopless 出力                   | main.go: return 0 always + ERROR:/OK: phase=end stop= |
| 互換テスト (旧 bundle → 新 verify PASS)       | contract_law_test.go                                  |

Discovery で確認済み：
- `ContractVersion: "1.1"` は create.go L57/L242 で既に設定
- `ValidateContractV11` は contract.go で既に実装
- `VerifyBundle` は verify.go で既に実装 (500行の堅牢なverifier)
- 今回の主な追加: 文書化 + CLI stopless化 + バックワード互換テスト

## Verification

```
go build ./cmd/reviewbundle        → OK: build clean
go test ./cmd/reviewbundle/... -run "TestVerify_Stopless|TestVerify_ContractV11|TestVerify_ContractVersion" -v
  → PASS: TestVerify_StoplessOutput (missing_path_arg, nonexistent_bundle)
  → PASS: TestVerify_ContractV11Compat
  → PASS: TestVerify_ContractVersionSingleSource
rg "os\.Exit|log\.Fatal|log\.Panic" cmd/reviewbundle/*.go
  → main.go:11 os.Exit(run(...)) のみ (entrypoint pattern, acceptable)
```

## Evidence

- Build: clean
- Tests: 3 new tests PASS
- Audit: os.Exit in main() only (entrypoint, unavoidable in Go)
- CONTRACT doc: docs/ops/REVIEW_BUNDLE_CONTRACT_v1.md
