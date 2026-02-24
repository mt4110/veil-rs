# PR-91 — S12-06A: verify chain hardening（検証連鎖の完全整流）

## SOT

- Scope: S12-06A — verify chain hardening (STDOUT_CONTRACT doc + chain tests + main.go full stopless)
- Branch: s12-06-verify-chain-hardening-v1
- **Auto-merge disabled — PR intentionally left open for review.**
- Deliverables:
  - docs/ops/STDOUT_CONTRACT_v1.md (new: canonical stdout protocol)
  - cmd/reviewbundle/chain_test.go (new: 3 chain integration tests)
  - cmd/reviewbundle/main.go (fully stopless: default/no-arg now emit phase=end)

## What

S12-06A は prverify / reviewbundle / flake / localgc の stop解釈を連鎖全体で整流する。

| DoD                                       | 実装                                     |
| ----------------------------------------- | ---------------------------------------- |
| 全入口/consumer の stop解釈が文書化       | docs/ops/STDOUT_CONTRACT_v1.md           |
| exit-code で成功扱いが起きないテスト      | cmd/reviewbundle/chain_test.go (3 tests) |
| どの入口も OK: phase=end stop= を必ず出す | main.go default/no-arg case 修正         |

Chain audit 結果（S12-05.6 + B/C で既に整備済み）:

```
Entrypoints emitting OK: phase=end stop=:
  ✓ cmd/prverify/main.go
  ✓ cmd/reviewbundle/main.go  (verify + create + default + no-arg)
  ✓ cmd/reviewbundle/create.go
  ✓ cmd/localgc/main.go

Consumers parsing stop=:
  ✓ flake.nix L196-199        (grep -qF "OK: phase=end stop=0")
  ✓ create.go L332-336        (strings.Contains "OK: phase=end stop=0")
  ✗ no remaining exit-code consumers found
```

## Verification

```
go build ./cmd/reviewbundle         → OK: build clean
go test ./cmd/reviewbundle/... -run TestChain -v
  → PASS: TestChainViolationYieldsStop1 (nonexistent_bundle, missing_bundle_arg)
  → PASS: TestChainSuccessYieldsStop0   (stop_values_mutually_exclusive)
  → PASS: TestChainPhaseEndAlwaysLast   (verify/unknown/no-arg all end with phase=end)
```

## Evidence

- Build: clean
- 3 chain integration tests PASS
- main.go: all code paths now emit OK: phase=end stop= (fully stopless)
- STDOUT_CONTRACT_v1.md: documents stop flag, line types, entrypoints, consumers
