# PR-46: Strict Exceptions CLI Flags & strict_lock_busy 復活

## Source of Truth (SOT)
📋 **SOT Document**: [`docs/pr/PR-46-v0.22.3-epic-d-strict-exceptions-cli-lock.md`](docs/pr/PR-46-v0.22.3-epic-d-strict-exceptions-cli-lock.md)

---

## RunAlways Verification (入口固定)
All CI and local verification uses these two commands:

```bash
cargo test --workspace
nix run .#prverify
```

---

## Evidence (証拠チェーン)

### Baseline (Green)
- **Commit**: `967e8a3`
- **prverify**: `.local/prverify/prverify_20260209T215359Z_967e8a3.md` ✅

### Current (最新)
- **Commit**: `<FILL: git rev-parse --short=7 HEAD>`
- **prverify**: `<FILL: .local/prverify/prverify_YYYYMMDDTHHMMSSZ_<sha>.md>` ✅

---

## Change Summary (変更点)

This PR delivers:

- ✅ **CLI Wiring**: `veil exceptions` subcommand now reachable from CLI
- ✅ **`--strict-exceptions`**: Fail fast on missing/invalid/expired registry (deterministic exit codes)
- ✅ **`--system-registry`**: Force system-wide registry path
- ✅ **Flag Exclusivity**: `--system-registry` + explicit path → immediate error with recovery message
- ✅ **Lock Busy Proof**: Tests prove non-blocking behavior (no hangs), error messages include recovery steps
- ✅ **Message Contracts**: All strict-mode errors follow "what/where/why/next" pattern

---

## Contracts (不変条件)

### Error Message Contract (1-scroll recovery)
All strict-mode errors include:
- **What**: Conclusion (what happened)
- **Where**: Path/Resource
- **Why**: Reason
- **Next**: Fix command or recovery step

### Flag Exclusivity Contract
Simultaneous use of `--system-registry` and explicit registry path:
- **Result**: Immediate error (exit code != 0)
- **Message**: "Cannot use both --system-registry and explicit path. Use one or the other."
- **Examples**: Both valid alternatives shown

### Lock Busy Contract
When lock is held by another process:
- **strict mode**: Immediate failure (no wait)
- **Message**: "Lock held by another process. Retry: `<command>`"
- **Test**: Proves non-blocking (timeout-based hang detection)

---

## Commits (順序)
1. `docs: add PR46 description template`
2. `feat(cli): wire exceptions subcommand`
3. `feat(cli): add --strict-exceptions and enforce registry flag exclusivity`
4. `test: prove lock busy is non-blocking and message contract`
5. `docs: update PR46 SOT evidence and contracts`

---

## Pre-Merge Checklist

- [ ] `cargo test --workspace` ✅
- [ ] `nix run .#prverify` ✅
- [ ] `veil --help` shows `exceptions`
- [ ] `veil exceptions --help` shows `--strict-exceptions` and `--system-registry`
- [ ] Both flags together → immediate error with contract message
- [ ] Lock busy test passes without hanging (local + CI)
- [ ] No blocking locks introduced (`try_lock_*` only)
- [ ] SOT evidence updated with final commit + prverify report
