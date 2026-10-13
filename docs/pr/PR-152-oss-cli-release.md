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
- Declare `MIT OR Apache-2.0` in workspace package metadata and inherit it in veil-cli, veil-core, veil-config, and veil-lsp. Existing license texts and legacy server/guardian/pro manifests remain unchanged.

- Refresh stale release/README/Linux documentation for the Rust 1.92.0 Linux/macOS CI, two-platform RC scope, and macOS ad-hoc trial limitations. Final ZIPs and whole-archive SHA256 are prepared from the final documentation commit in new external output paths.

- Validate local signed RulePack contents in sibling staging before atomic directory publication; reject invalid/duplicate rule contents and keep failed installations retryable. Explicit activation instructions preserve existing configuration. Give each CI profile a unique artifact name.

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

The CI/documentation follow-up `5cd23e9` passed all seven hosted checks: [CI](https://github.com/mt4110/veil-rs/actions/runs/38398797938) and [SOT Guard](https://github.com/mt4110/veil-rs/actions/runs/38398797593).

The license-metadata follow-up adds no runtime source or dependency changes. Local Cargo metadata resolves the four supported crates to `MIT OR Apache-2.0`; third-party inventory, release conditions, architecture, document taxonomy, and whitespace checks passed before push. `cargo package --list --allow-dirty --locked --offline` was inspected for all four crates: it does not include the repository-root license originals. CLI ZIP packaging includes both originals; crates.io source-package license inclusion remains a separate gate before any package publication. The metadata head passed all seven checks: [CI](https://github.com/mt4110/veil-rs/actions/runs/38400609392) and [SOT Guard](https://github.com/mt4110/veil-rs/actions/runs/38400609028).

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

The documentation head `b07b822` passed all seven checks and both native ZIP verifications. Final merge review found RulePack installation validation/retry defects and duplicate artifact names. The follow-up fixes are covered by RulePack integration regressions; new hosted CI and fresh two-platform ZIP verification must pass before merging or creating the release draft. Previous ZIPs are retained as historical candidates.

Local follow-up verification: RulePack integration tests passed in default and minimal configurations (8 tests each; overlapping tests). Clippy for veil-cli all targets, Python 85 tests, architecture/release/license/document guards, actionlint, and git diff whitespace checks passed. No dependencies or Cargo.lock changes.

An explicit RulePack expiry is enforced whenever the normal loader is invoked, not only during installation. Packs without an expiry remain compatible. Core coverage checks expired/future/legacy TTL states; CLI regressions check expired configured rules fail with exit code 2 and preserve installed bytes.

Expiry follow-up local checks passed: core expiry regression, 9 CLI RulePack integration tests in each default/minimal configuration, and core/CLI all-targets Clippy. Fresh final ZIPs must include this follow-up.

RulePack placement rejects a symlink or file at `.veil` before staging or publishing, and requires the canonical installation parent to remain beneath the canonical project directory. Unix regressions preserve both existing and dangling symlinks and outside-directory sentinel files.

Placement follow-up local checks passed: 10 RulePack CLI integration tests in each default/minimal configuration and veil-cli all-targets Clippy. All new candidate ZIPs are regenerated from the final placement-fix head; intermediate ZIPs remain historical artifacts.
