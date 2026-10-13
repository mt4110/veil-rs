---
release: 0.21.0-rc.1
pr: 152
status: Review
created_at: 2026-10-13
branch: release/0.21.0-rc.1-linux-validated
title: Reject incomplete scans and prepare the local OSS CLI
---

# PR #152: local OSS CLI release candidate

## SOT

The supported product is the local Rust CLI described in [local OSS architecture](../architecture/local-oss.md).
[Release conditions](../ci/oss-cli-release.md) define the distribution gates.
The previously validated source candidate is `df2fae53c4432e885ca3d0b7e1869d39fa86dd8d`; subsequent CI and documentation changes need fresh hosted verification.

## What

- Preserve incomplete-scan errors after baseline suppression and scan the first Git commit.
- Reject partial baseline writes, interactive read failures, and missing Git blobs while preserving existing baselines.
- Align CLI version, public architecture, distribution contents, and third-party notices for 0.21.0-rc.1.
- Add this PR record without weakening SOT Guard.
- Limit this release's CI matrix to Linux and macOS. Windows remains outside verification and distribution scope pending a native test environment.
- Name native CI jobs for Rust 1.92.0 and request that exact toolchain, matching the repository pin. This is not latest-stable coverage.
- Clarify Japanese and English README headings around local detection and masking.

## Verification

Historical hosted checks for `df2fae53` passed Linux/MSRV/musl/macOS and common guards; two checks failed.
Windows stopped before the CLI build because `scripts/check_architecture.py` used the default CP1252 decoder for UTF-8 source text.
SOT Guard rejected the absence of a PR-specific changed document under `docs/pr/`.
Removing Windows from the supported matrix does not fix or validate Windows behavior.

Local follow-up checks on 2026-10-13 passed:

- `python3 scripts/check_architecture.py`: public boundaries and dependency graphs.
- `python3 scripts/check_cli_release.py`: CLI release conditions.
- `python3 -m unittest discover -s scripts -p 'test_*.py'`: 85 tests.
- `python3 scripts/check_docs_taxonomy.py`: all 555 public documents classified.
- `actionlint` on the CI, reusable OSS profile, and SOT Guard workflows.
- `git diff --check` and the SOT changed-file existence condition (including the new untracked PR record).

No Rust runtime source changed in this follow-up; no full Rust rebuild was run here. Hosted CI on a new head remains pending until the changes are committed and pushed.

## Evidence

- [PR #152](https://github.com/mt4110/veil-rs/pull/152)
- [Historical CI run](https://github.com/mt4110/veil-rs/actions/runs/38347389836)
- [Historical SOT Guard run](https://github.com/mt4110/veil-rs/actions/runs/38347389670)
- [Release verification scope and limitations](../ci/oss-cli-release.md)

## Non-goals

Windows support, latest-stable coverage, telemetry, database changes, new runtime services, merging, tagging, and publishing are outside this follow-up.
macOS Developer ID signing, notarization, Gatekeeper download testing, Intel Mac, and additional Linux architectures remain unverified.

## Rollback

Revert the follow-up commit if needed; retain the previously validated source candidate and its verification artifacts.
