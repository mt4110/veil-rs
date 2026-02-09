# Registry Lock Contention (Non-Blocking Behavior)

## Overview

Since PR45, veil uses **non-blocking file locks** for exception registry access. If another process holds the lock, registry operations fail immediately instead of blocking.

## Behavior (PR45 - Core Only)

### Current Implementation

When `Registry::load()` or `Registry::save()` encounters a lock held by another process:

- **Returns**: `RegistryError::LockBusy(PathBuf)` immediately (no waiting)
- **Lock file**: `.veil/exception_registry.lock` (created alongside `.veil/exception_registry.toml`)

### Impact on CLI (Current)

The current `veil scan` command:
- Warns about lock contention
- Continues scan without exception registry
- Exits based on scan findings (not registry errors)

> **Note**: Deterministic exit codes for lock errors require `--strict-exceptions` flag (planned for PR46).

## Troubleshooting

### Symptom: "Registry is locked by another process"

**Common Causes**:
1. Multiple `veil scan` processes running concurrently (CI parallel jobs)
2. Long-running scan holding the lock
3. Stale lock file from crashed process

**Recovery**:

```bash
# Check for running veil processes
ps aux | grep veil

# If no processes found, remove stale lock file
rm -f .veil/exception_registry.lock

# Re-run scan
veil scan .
```

### Preventing Lock Contention in CI

**Option 1**: Sequential execution (simple)
```yaml
# Don't run veil in parallel jobs that share the same workspace
jobs:
  security-scan:
    runs-on: ubuntu-latest
    steps:
      - run: veil scan .
```

**Option 2**: Per-job registry (advanced)
```yaml
# Each job uses its own registry path
jobs:
  scan-frontend:
    steps:
      - run: veil scan frontend/ --system-registry .veil/frontend-registry.toml
  scan-backend:
    steps:
      - run: veil scan backend/ --system-registry .veil/backend-registry.toml
```

> **Note**: `--system-registry` flag requires PR46.

## Future (PR46)

With `--strict-exceptions` mode:
- Lock errors will exit code 2 (instead of warn)
- Error message includes lock path and recovery command
- Operators can enforce strict registry consistency in CI gates

## Technical Details

**Lock Types**:
- `Registry::load()`: Shared lock (`try_lock_shared`) - allows multiple readers
- `Registry::save()`: Exclusive lock (`try_lock_exclusive`) - single writer

**Error Mapping**:
- `std::io::ErrorKind::WouldBlock` → `RegistryError::LockBusy`
- `std::io::ErrorKind::PermissionDenied` → `RegistryError::PermissionDenied`
- Other errors → `RegistryError::Io`

## Related

- Registry format: See `docs/runbook/exception-registry.md` (prverify registry)
- Strict mode: Planned for PR46
