# Implementation Plan - v0.4 Epic D: DX Improvements (Global Flags)

**Goal**: Polish the CLI experience by adding standard global flags for coloring and quiet execution.

## Proposed Changes

### crate: veil-cli

#### [MODIFY] [cli.rs](crates/veil-cli/src/cli.rs)
- Add global flags to `Cli` struct:
    - `--no-color`: Disable colored output.
    - `--quiet`: Suppress non-essential output (logs, progress).

#### [MODIFY] [main.rs](crates/veil-cli/src/main.rs)
- **Warning/Error Handling**:
    - If `--quiet` is set, set `RUST_LOG=error` (unless overridden) and disable progress bars.
- **Color Handling**:
    - If `--no-color` is set, call `colored::control::set_override(false)`.

## Verification Plan
1. **No Color**: `veil scan --no-color` -> Verify output has no ANSI codes.
2. **Quiet**: `veil scan --quiet` -> Verify no progress bar or info logs, only JSON/Table output.

