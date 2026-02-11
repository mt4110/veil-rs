# PR-56 Implementation Plan — Issue #47 PR-C (idna/url)
Slug: issue-47-prc-idna-url-remediation

## Goal
Close the `idna/url` dependabot alert with **minimal dependency movement**, and leave a deterministic evidence trail.

## Constraints (non-negotiable)
- No new `*.sh` files (pre-commit guard)
- Prefer **lockfile-only** updates
- Keep the diff explainable in 1 scroll
- Always Run Contract must PASS

## Normative target (security)
- If `idna` is direct dependency: upgrade to `idna >= 1.0.3`
- If `idna` comes via `url`: upgrade to `url >= 2.5.4`

Refs:
- https://rustsec.org/advisories/RUSTSEC-2024-0421
- https://github.com/advisories/GHSA-h97m-ww89-6jmq

---

## Plan (deterministic steps)

### P0. Preflight baseline (must be reproducible)
1) Ensure clean working tree
- `git status --porcelain=v1`

2) Snapshot Issue #47 into SOT (verbatim)
- `gh issue view 47 --json title,body,state,url`
- `gh issue view 47 --comments --json comments | jq -r '.comments[].body' | sed -n '1,200p'`

3) Snapshot dependency graph BEFORE (into SOT)
- `cargo tree -i idna`
- `cargo tree -i url`
- `cargo tree -p url`
- `cargo tree -p idna`

### P1. Determine the parent chain (choose one path)
Decision rule:
- IF `cargo tree -i idna` shows `url` in the parent chain
  - choose **Path-A** (bump `url`)
- ELSE IF any workspace crate directly depends on `idna`
  - choose **Path-B** (bump `idna`)
- ELSE
  - treat as investigation failure; do not proceed without capturing evidence in SOT

### P2. Apply minimal update (prefer lockfile-only)
#### Path-A (expected): bump url
1) Try minimal precise bump:
- `cargo update -p url --precise 2.5.4`

2) Verify resulting versions:
- `cargo tree -p url`
- `cargo tree -p idna`
- `cargo tree -i idna`

#### Path-B: bump idna directly
1) Try minimal precise bump:
- `cargo update -p idna --precise 1.0.3`

2) Verify resulting versions:
- `cargo tree -p idna`
- `cargo tree -i idna`

Notes:
- Do NOT run workspace-wide `cargo update`
- Keep the bump minimal (exact version with `--precise`)

### P3. Verification (Always Run Contract)
- `cargo test --workspace`
- `nix run .#prverify`
- Save evidence: `docs/evidence/prverify/prverify_<UTC>_<sha7>.md`
- `cockpit check`

### P4. Documentation + Evidence pin
- Update SOT:
  - BEFORE/AFTER versions
  - chosen path and rationale
  - dependency snapshots AFTER
  - evidence path (prverify) + cockpit check snippet
  - 1-paragraph “why it closes the alert”

### P5. Final sanity
- Ensure changed files are only:
  - Cargo.lock (and only Cargo.toml if required)
  - docs/pr/*
  - docs/evidence/prverify/*
- Ensure no unrelated dependency drift

---

## IF_FAIL Playbook (fast + deterministic)

### IF `cargo update -p url --precise 2.5.4` updates too much
- Abort and reset lockfile changes
- Retry with:
  - confirm `Cargo.lock` diff scope
  - keep only url/idna-related minimal changes
- Record the failure + diff in SOT before retry

### IF tests or prverify fail
- Capture the failing output in evidence (do not summarize without logs)
- Identify the cause:
  - `rg -n "\burl::|Url\b" crates`
- Fix minimally OR choose alternate minimal version that still satisfies target
- Update SOT “Decision record” with the reason

### IF toolchain/MSRV is incompatible
- Capture exact error in evidence
- Prefer:
  - minimal compatible fixed version satisfying target
  - (only if policy allows) adjust toolchain explicitly and document why
