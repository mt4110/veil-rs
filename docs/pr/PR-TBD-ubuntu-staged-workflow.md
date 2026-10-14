---
status: Draft
created_at: 2026-10-14
branch: docs/ubuntu-staged-workflow
title: Ubuntu first scan, staged changes and repeat use
---

# Ubuntu first scan, staged changes and repeat use

## SOT

Use existing `veil scan` and `veil scan --staged` behavior. No scanner, config, CLI options, or masking contract changes. Target Ubuntu x86_64 using the published 0.21.0-rc.1 trial; retained work in the owner checkout remains outside this PR.

## What

- Connect the own-file first scan to deliberate staging, index inspection, staged scanning and repeat use before commit.
- Explain index vs working-copy contents, restaging fixes, full-file scanning, empty stages, unstaged/untracked/unchanged files, ignore rules and binary skips.
- Preserve the optional synthetic tutorial, existing config/baseline and macOS ad-hoc restrictions.
- Add a fake Git-project acceptance runner and run it on the release CLI used to create the archive in all native CI profiles; retain its small result record with existing verification artifacts.

## Verification

17 fake-project cases passed on the published macOS ARM64 RC binary: clean first scan, empty/new index, staged detection after a working-copy fix, restaging, repeated edits, exclusions, explicit ignored-file scanning, binary skips, missing input and execution outside Git. Each scan checks redaction and source/index/HEAD preservation. Synthetic keys are generated at runtime and are never real credentials.

The previously used Ubuntu host currently refuses SSH. Local macOS results are not Linux proof; updated Ubuntu/MSRV/musl CI must pass the same release-CLI workflow before completion. No source files or real credentials are uploaded by the runner.

## Evidence

Baseline main: `0d543683bb85330e31675e73a4dd197e03f5a844`.
CI records `staged-workflow.json` with case name, exit code, scanned/skipped counts and preservation result, without source or matched content.

## Non-goals

No automatic user-project hook installation, new telemetry, GUI, release changes, merge or publication. CI passing does not establish third-party first-use success or retention. Existing public ZIP contents are not changed by this documentation PR.
