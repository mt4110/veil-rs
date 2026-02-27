# Veil-rs Release Guide

This document outlines the standard release procedure for `veil-rs` to ensure predictability, stability, and confidence in every release.

## 1. Pre-Release Checks & Version Bump

1. **Bump Version**: Update the version number in the root `Cargo.toml`:
   ```toml
   [workspace.package]
   version = "1.0.0"
   ```
2. **Run Tests**: Ensure all tests pass.
   ```bash
   cargo test --workspace
   ```
3. **Run Benchmarks**: Verify there are no performance regressions ("基準: 遅くなってないこと").
   ```bash
   cargo bench
   ```
   *The scan time for standard payloads should remain consistent with previous releases.*

## 2. Local Environment Smoke Test

Before tagging, verify the built binary locally. This ensures no surprises during the actual release.

```bash
# 1. Build release binary
cargo build --release

# 2. Run the smoke test suite to verify exit codes, stdout purity, and core limits
./scripts/smoke_release.sh
```

## 3. Tagging the Release

Once you are confident in local tests:

```bash
# Commit the version bump
git add Cargo.toml Cargo.lock
git commit -m "chore: bump version to v1.0.0"

# Create an annotated tag
git tag -a v1.0.0 -m "Release v1.0.0"

# Push to trigger CI release workflow
git push origin HEAD --tags
```

> **Tip**: For release candidates, use tags like `v1.0.0-rc.1` to lock and test the exact source code before the final release.

## 4. Release Artifacts

The final GitHub Release should include:

1. **Release Notes**: E.g., copy from `v1.0.0_release_notes.md`.
2. **Binaries**: Pre-compiled artifacts for supported platforms.
3. **Checksums**: `SHA256SUMS` for the generated binaries to ensure bundle integrity.
