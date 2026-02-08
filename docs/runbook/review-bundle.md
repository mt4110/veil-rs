# Runbook: Review Bundle

This runbook describes how to generate a **review bundle** — a small archive optimized for fast, deterministic review.

A review bundle contains:
- `INDEX.md` (navigation)
- `meta/` (base/head, branch, changed file list, stats)
- `files/` (latest snapshots of changed files)
- `patch/` (committed patch series + optional WIP diffs)
- `evidence/` (single prverify PASS log when available)

---

## Quick Start

### 1. Generate Bundle (Clean)
Recommended for final PR submission. Requires a clean working tree.
```bash
bash ops/review_bundle.sh
```
Output: `.local/review-bundles/veil-rs_review_YYYYMMDD_HHMMSS_<commit>.tar.gz`

### 2. Generate Bundle (WIP)
Useful for pre-commit review or debugging locally. Includes staged/unstaged changes.
```bash
MODE=wip bash ops/review_bundle.sh
```
Output: `.local/review-bundles/veil-rs_review_wip_YYYYMMDD_HHMMSS_<commit>.tar.gz`

## Configuration (Optional)

You can customize behavior with environment variables:

| Variable | Default | Description |
| :--- | :--- | :--- |
| `MODE` | `clean` | `clean` (committed only) or `wip` (include dirty state) |
| `OUT_DIR` | `.local/review-bundles` | Directory to save the bundle |
| `INCLUDE_FILE` | `.review-bundle.include` | Path to file listing extra files to bundle |
| `BASE_REF` | `origin/main` | Diff base reference |

### Including Extra Files
To bundle additional context files (e.g., related docs or config) that haven't changed:
1. Create `.review-bundle.include` in the repo root.
2. Add file paths (one per line). Supports simple globs.
   ```text
   # Example .review-bundle.include
   docs/runbook/*.md
   ops/config.toml
   ```

## Reviewing a Bundle

1. **Extract**:
   ```bash
   mkdir review
   tar -xzf veil-rs_review_*.tar.gz -C review
   ```
2. **Read INDEX**:
   Open `review/INDEX.md` to see metadata and navigation.
   ```bash
   cat review/INDEX.md
   ```

3. **Inspect**:
   - `files/`: Contains the full content of changed files (snapshot).
   - `patch/`: Contains `.patch` files for applying or viewing diffs.
   - `evidence/`: Contains the verification log. **Verify one exists!**
     ```bash
     ls review/evidence/
     ```
