# PR-TBD-v1: Epic A S10-08 prkit contract tests

## Scope
- Contract tests for `prkit` CLI
- Refactor `cmd/prkit/main.go` for testability (dependency injection of stdout/stderr)
- Deterministic output guarantees (no time/path/env dependency in output)

## Contract Summary (S10-07 preserved)
The following behaviors are locked by contract tests:

### 1. Help
- **Command**: `prkit --help`
- **Exit Code**: `0`
- **Stdout**: Empty
- **Stderr**: Contains usage information. Deterministic (no timestamps).

### 2. Unknown Flag
- **Command**: `prkit --unknown-flag`
- **Exit Code**: `2`
- **Stdout**: JSON error object (portable-json format).
  ```json
  {"error":"flag provided but not defined: -unknown-flag"}
  ```
- **Stderr**: Contains "flag provided but not defined" and usage information.

### 3. Execution (SOT)
- **Command**: `prkit --sot-new` ...
- **Exit Code**: `0` (success) or `1` (runtime error) or `2` (usage error)
- **Stdout**: portable-json if error, otherwise silent/structured.

## Evidence
- Report: [docs/evidence/prverify/prverify_20260214T115121Z_c0a1a07.md](../evidence/prverify/prverify_20260214T115121Z_c0a1a07.md)
- Tests: `go test -count=1 ./cmd/prkit`
- Verification: `nix run .#prverify`

## Changes
- `cmd/prkit/main.go`: Refactor to `Run(args, stdout, stderr)`
- `cmd/prkit/main_test.go`: Add contract tests
- `docs/ops/S10_08_PLAN.md`: Plan
- `docs/ops/S10_08_TASK.md`: Logic
