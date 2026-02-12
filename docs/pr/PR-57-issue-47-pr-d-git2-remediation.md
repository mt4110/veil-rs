# PR-57 Issue #47 PR-D (git2) Remediation

## Objective
Fix Dependabot `git2` alerts (GHSA-j39j-6gw9-jw6h) by ensuring `git2 >= 0.20.4`.
Previous closure of Issue #47 was premature as Dependabot lag or lockfile resolution was incomplete.
This PR explicitly forces `git2` to `0.20.4` in `veil-cli` and resolves the lockfile.

## Changes
- `crates/veil-cli/Cargo.toml`: `git2` updated from `0.19` to `0.20.4`.
- `Cargo.lock`: `git2` updated to `0.20.4` (along with `libgit2-sys` and `libssh2-sys` updates).

## Verification (Always Run Contract)

### 1. Evidence Snapshots
- **PRE**: `20260212T040957Z`
  - Dependabot: [json](../../evidence/dependabot/pr57_git2_alerts_pre_20260212T040957Z.json) / [md](../../evidence/dependabot/pr57_git2_alerts_pre_20260212T040957Z.md)
  - Cargo tree: [txt](../../evidence/deps/pr57_cargo_tree_git2_pre_20260212T040957Z.txt)
  - Cargo lock: [txt](../../evidence/deps/pr57_cargolock_git2_pre_20260212T040957Z.txt)

- **POST**: `20260212T041559Z`
  - Dependabot: [json](../../evidence/dependabot/pr57_git2_alerts_post_20260212T041559Z.json) / [md](../../evidence/dependabot/pr57_git2_alerts_post_20260212T041559Z.md)
  - Cargo tree: [txt](../../evidence/deps/pr57_cargo_tree_git2_post_20260212T041559Z.txt)
  - Cargo lock: [txt](../../evidence/deps/pr57_cargolock_git2_post_20260212T041559Z.txt)

### 2. prverify
- Command: `nix run .#prverify`
- Result: **PASS**
- Report: [prverify_20260212T050653Z_c4ef971.md](../../evidence/prverify/prverify_20260212T050653Z_c4ef971.md)

### 3. Cockpit Check
- Command: `nix run .#check`
- Result: **PASS**

```bash
Running check with go version go1.24.11 darwin/arm64
COCKPIT_RFv1: RESULT=PASS STEP=check

✓ cockpit check passed
```
