# Runbook: Review Bundle

This runbook describes how to generate a **review bundle** — a small archive optimized for fast review.

A bundle contains:
- `INDEX.md` (navigation)
- `meta/` (base/head, branch, changed file list, stats)
- `files/` (latest snapshots of changed files)
- `patch/` (committed patch series + optional WIP diffs)
- `evidence/` (prverify PASS log when available)

---

## Quickstart

From repository root:

```bash
bash ops/ci/review_bundle.sh
```

This generates a clean bundle under:

- `.local/review-bundles/<project>_review_<timestamp>_<head12>.tar.gz`

> `<project>` is derived from the repo root directory name.

---

## Pre-commit review (WIP)

To review changes **before committing**:

```bash
MODE=wip bash ops/ci/review_bundle.sh
```

This generates:

- `.local/review-bundles/<project>_review_wip_<timestamp>_<head12>.tar.gz`

WIP mode additionally includes:
- `patch/wip_index.patch` (staged diff)
- `patch/wip_worktree.patch` (unstaged diff)
- untracked files copied into `files/` (if relevant)

---

## Inspecting a bundle

Extract:

```bash
tar -xzf .local/review-bundles/*_review*.tar.gz
# Extracts into ./review/
```

Read the index first:

```bash
sed -n '1,200p' review/INDEX.md
```

Common reading order:
1) `review/meta/changed_files.txt`
2) `review/files/`
3) `review/patch/series.patch`
4) `review/evidence/` (if present)

---

## Configuration

Environment variables:

- `REPO` (default: `.`)
  - repository path

- `MODE` (default: `clean`)
  - `clean` or `wip`

- `OUT_DIR` (default: `.local/review-bundles`)
  - output directory (relative to repo root or absolute)

- `BASE_REF` (default: `origin/main`)
  - merge-base ref used to compute `base`

- `INCLUDE_FILE` (default: `.review-bundle.include`)
  - optional file listing extra files to always include (one pathspec per line)
  - supports simple globs like `docs/runbook/*.md`
  - blank lines and `# comments` are ignored

- `EVIDENCE_FILE` (default: unset)
  - explicitly include a prverify PASS log (relative to repo root or absolute)

Examples:

```bash
BASE_REF=main bash ops/ci/review_bundle.sh
OUT_DIR=/tmp/review-bundles MODE=wip bash ops/ci/review_bundle.sh
EVIDENCE_FILE=.local/prverify/prverify_20260208T075813Z_abcdef0.md bash ops/ci/review_bundle.sh
```

---

## Evidence behavior

Evidence is optional. The tool will attempt to include:

1) a PASS log matching `HEAD` (12 or 7 short SHA), otherwise  
2) the latest `prverify_*.md`, and write a warning trail.

If `review/evidence/` is empty, check:

- `review/meta/warnings.txt`

---

## Security notes

Do not generate or share a bundle that contains secrets.
