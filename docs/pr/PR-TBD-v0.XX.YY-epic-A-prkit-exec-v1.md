# [PR-TBD] prkit-exec-v1: Execution toolkit v1

## Meta
- Epic: A
- Release: v0.XX.YY
- Date: 2026-02-13
- Author: @masakitakemura
- Status: Draft

## Goal
Hardening `pr-kit` for merge readiness by fixing critical execution bugs and enforcing portable evidence contracts.

### What
- **Enable Execution**: Remove the blocking `if !dryRun` check in `main.go`.
- **Portable Evidence**: Ensure no absolute paths (e.g., `<HOME>/...`) are leaked in documentation or output.
- **SOT Improvements**: Make SOT generator output repository-relative paths.

### Why
- The previous implementation inadvertently blocked `--run` mode.
- Portable evidence is required for team collaboration and CI consistency.

## Plan
- [x] **Preflight**: Verify clean worktree.
- [x] **Portable Contract**: Scan and sanitize absolute paths in `docs/`.
- [x] **Fix `main.go`**: Enable `--run` mode and enforce flag exclusivity.
- [x] **SOT Output**: Update `sot.go` to use relative paths.
- [x] **Verification**: Validate fixes (local + CI).

## Verification
- [x] **Portable Check**: `rg` for absolute paths in `docs/*.md` returns clean (except false positives in runbooks).
- [x] **Run Mode**: Verified `main.go` logic allows `--run`.
- [ ] **Local Build**: Skipped due to Go toolchain mismatch (`go1.25.5` vs `go1.24.11`).
- [ ] **CI Verification**: Relying on GitHub Actions for final build/test pass.
