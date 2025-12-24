# Cockpit Single Source of Truth (SSOT)

This document serves as the authoritative source for Cockpit rules, contracts, and operational requirements.

## Git Required

**Git is a hard requirement** for all Cockpit commands and build operations.

*   **Reason**: We rely on `git describe`, dirty tree detection, and commit hashing for version stamping and metrics.
*   **Parity**: CI and Local environments must both have `git` available.
    *   **CI**: `actions/checkout` with `fetch-depth: 0` (full history) and `clean: true`.
    *   **Nix**: `git` is included in `runtimeInputs` for all Cockpit apps (`check`, `go-test`, etc.).
*   **Dirty Tree**:
    *   **CI**: Must be clean.
    *   **Local**: Allowed, but may trigger warnings or specific metric flags (`dirty: true`).

## Clean Tree
<a id="clean-tree"></a>
A "Clean Tree" means no uncommitted changes (tracked or untracked).
Cockpit commands may enforce this in CI environments to ensure reproducibility.

## Flake Lock Policy

*   **Updates**: Explicit updates only (e.g. `nix flake update`).
*   **CI**: CI does NOT auto-update `flake.lock`.
