# PR-45: Registry Lock Fix (fs2 Non-Blocking)

## Goal

Fix registry lock handling to use **non-blocking file locks** via `fs2`, preventing hangs when multiple `veil` processes access the registry concurrently.

## Scope (Core Only - PR45)

* ✅ Add `fs2` dependency to workspace
* ✅ Update `registry.rs` to use `fs2::FileExt::try_lock_*` 
* ✅ Export `Registry` and `FindingId` from `veil-core` public API
* ✅ Handle `WouldBlock` error as `RegistryError::LockBusy`
* ✅ Operations runbook for lock contention scenarios

## Deferred to PR46

* ❌ CLI flags: `--system-registry`, `--strict-exceptions`
* ❌ Deterministic exit codes (2 = registry error, 3 = expired exceptions)
* ❌ Integration tests for strict mode behavior
* ❌ `veil exceptions` subcommands

## Implementation

### Core Changes

**`crates/veil-core/src/registry.rs`**:
- Replace blocking locks with `fs2::FileExt::try_lock_shared()` and `try_lock_exclusive()`
- Map `ErrorKind::WouldBlock` to `RegistryError::LockBusy`
- Preserve existing `PermissionDenied` and `Io` error mapping

**`crates/veil-core/src/lib.rs`**:
- Add `pub mod finding_id;` and `pub mod registry;`
- Export `pub use finding_id::FindingId;`
- Export `pub use registry::Registry;`

### Why Non-Blocking?

**Problem**: Blocking locks cause hangs in CI parallel jobs or when multiple developers run `veil scan` concurrently.

**Solution**: Non-blocking locks fail fast with `WouldBlock`, allowing callers to decide recovery (retry, skip, fail).

## Operations Runbook

See [`docs/ops/strict_exceptions_recovery.md`](../ops/strict_exceptions_recovery.md) for lock contention troubleshooting.

**Lock Contention Behavior** (PR45):
- Registry operations return `RegistryError::LockBusy` immediately if lock is held
- Current CLI behavior: warns and continues without exception registry
- **Future** (PR46): `--strict-exceptions` will exit with code 2

## Verification

```bash
# Core tests pass with new lock behavior
nix develop -c cargo test -p veil-core

# CLI builds without errors
nix develop -c cargo build -p veil-cli
```

## Related

- **PR44**: Exception Registry UX (base branch if not merged yet)
- **PR46** (future): CLI strict mode flags and exit codes
