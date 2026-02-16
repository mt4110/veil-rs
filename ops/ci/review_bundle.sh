#!/usr/bin/env bash
set -euo pipefail

# Review bundle generator (repo-agnostic / deterministic-ish / human-friendly)
#
# Modes:
#   MODE=clean (default): requires clean working tree; bundles committed range base..HEAD
#   MODE=wip             : allows dirty tree; also bundles staged/unstaged/untracked changes
#
# Output:
#   default OUT_DIR: .local/review-bundles (inside repo toplevel)
#
# Optional extra include list:
#   INCLUDE_FILE (default): .review-bundle.include
#   - one pathspec per line (supports simple globs like docs/runbook/*.md)
#   - blank lines and comments (# ...) are ignored
#
# Evidence:
#   - If EVIDENCE_FILE is set, it is included (absolute or relative to repo root)
#   - Otherwise auto-detect .local/prverify:
#       1) Prefer logs matching HEAD (12 or 7 short SHA)
#       2) Fallback to latest prverify_*.md with a warning trail
#
# Usage (from repo root):
#   bash ops/review_bundle.sh
#   MODE=wip bash ops/review_bundle.sh
#
# Optional:
#   REPO=../some-repo BASE_REF=origin/main OUT_DIR=.local/review-bundles MODE=wip bash ops/review_bundle.sh

repo="${REPO:-.}"
BASE_REF="${BASE_REF:-origin/main}"
MODE="${MODE:-clean}"
OUT_DIR="${OUT_DIR:-.local/review-bundles}"
INCLUDE_FILE="${INCLUDE_FILE:-.review-bundle.include}"
ts="$(date +'%Y%m%d_%H%M%S')"

cleanup() {
  if [ -n "${tmp:-}" ] && [ -d "${tmp:-}" ]; then
    rm -rf "$tmp"
  fi
}
trap cleanup EXIT

if [ "$MODE" != "clean" ] && [ "$MODE" != "wip" ]; then
  echo "ERROR: MODE must be 'clean' or 'wip' (got: $MODE)" >&2
  exit 1
fi

toplevel="$(git -C "$repo" rev-parse --show-toplevel)"
project="$(basename "$toplevel")"
project_slug="$(printf "%s" "$project" | tr -c 'A-Za-z0-9._-' '_' )"

# Clean requirement only for MODE=clean
if [ "$MODE" = "clean" ]; then
  if [ -n "$(git -C "$toplevel" status --porcelain=v1)" ]; then
    echo "ERROR: working tree is not clean. Commit or stash (including untracked) for deterministic bundle." >&2
    echo "Hint: run 'MODE=wip bash ops/review_bundle.sh' for pre-commit review." >&2
    exit 1
  fi
fi

head12="$(git -C "$toplevel" rev-parse --short=12 HEAD)"
head12="$(git -C "$toplevel" rev-parse --short=12 HEAD)"
head7="$(git -C "$toplevel" rev-parse --short=7 HEAD)"

# Anchor (last non-doc commit) for evidence stability
anchor_sha="$(git -C "$toplevel" log -1 --format=%H -- . ':(exclude)docs/**' 2>/dev/null || git -C "$toplevel" rev-parse HEAD)"
anchor7="$(printf "%s" "$anchor_sha" | cut -c1-7)"

# Resolve base (fallback chain)
if git -C "$toplevel" rev-parse --verify -q "$BASE_REF" >/dev/null; then
  base="$(git -C "$toplevel" merge-base HEAD "$BASE_REF")"
elif git -C "$toplevel" rev-parse --verify -q main >/dev/null; then
  base="$(git -C "$toplevel" merge-base HEAD main)"
else
  base="$(git -C "$toplevel" merge-base HEAD master)"
fi

suffix=""
if [ "$MODE" = "wip" ]; then
  suffix="_wip"
fi

# output dir (absolute or relative to repo root)
case "$OUT_DIR" in
  /*) out_dir="$OUT_DIR" ;;
  *)  out_dir="$toplevel/$OUT_DIR" ;;
esac
mkdir -p "$out_dir"

out="$out_dir/${project_slug}_review${suffix}_${ts}_${head12}.tar.gz"

tmp="$(mktemp -d)"
root="$tmp/review"
mkdir -p "$root/meta" "$root/patch" "$root/files" "$root/evidence"

# Always create warnings.txt so reviewers can rely on it existing.
: > "$root/meta/warnings.txt"

# --- META ---
git -C "$toplevel" rev-parse HEAD > "$root/meta/head_sha.txt"
echo "$base" > "$root/meta/base_sha.txt"
git -C "$toplevel" rev-parse --abbrev-ref HEAD > "$root/meta/branch.txt"
git -C "$toplevel" status --porcelain=v1 > "$root/meta/status.txt"
echo "$MODE" > "$root/meta/mode.txt"
echo "$BASE_REF" > "$root/meta/base_ref.txt"
echo "$OUT_DIR" > "$root/meta/out_dir.txt"
echo "$project" > "$root/meta/project.txt"

git -C "$toplevel" show -s --format='commit=%H%nshort=%h%ndate=%cI%nauthor=%an <%ae>%nsubject=%s' HEAD \
  > "$root/meta/head_commit.txt"

git -C "$toplevel" diff --stat "$base..HEAD" > "$root/meta/diff_stat_committed.txt"
git -C "$toplevel" diff --name-status "$base..HEAD" > "$root/meta/name_status_committed.txt"

if [ "$MODE" = "wip" ]; then
  git -C "$toplevel" diff --stat --cached > "$root/meta/diff_stat_index.txt"
  git -C "$toplevel" diff --name-status --cached > "$root/meta/name_status_index.txt"

  git -C "$toplevel" diff --stat > "$root/meta/diff_stat_worktree.txt"
  git -C "$toplevel" diff --name-status > "$root/meta/name_status_worktree.txt"

  git -C "$toplevel" ls-files --others --exclude-standard > "$root/meta/untracked_files.txt"
fi

# --- CHANGED FILE LIST (stable union) ---
{
  git -C "$toplevel" diff --name-only "$base..HEAD"
  if [ "$MODE" = "wip" ]; then
    git -C "$toplevel" diff --name-only --cached
    git -C "$toplevel" diff --name-only
    git -C "$toplevel" ls-files --others --exclude-standard
  fi
} | sed '/^$/d' | sort -u > "$root/meta/changed_files.txt"

# --- PATCH ---
git -C "$toplevel" format-patch --stdout "$base..HEAD" > "$root/patch/series.patch" || true
if [ "$MODE" = "wip" ]; then
  git -C "$toplevel" diff --cached > "$root/patch/wip_index.patch" || true
  git -C "$toplevel" diff > "$root/patch/wip_worktree.patch" || true
fi

# --- FILES (latest snapshots of changed files) ---
while IFS= read -r f; do
  [ -z "$f" ] && continue
  [ -f "$toplevel/$f" ] || continue
  mkdir -p "$root/files/$(dirname "$f")"
  cp "$toplevel/$f" "$root/files/$f"
done < "$root/meta/changed_files.txt"

# --- OPTIONAL EXTRA INCLUDE LIST (.review-bundle.include) ---
case "$INCLUDE_FILE" in
  /*) include_path="$INCLUDE_FILE" ;;
  *)  include_path="$toplevel/$INCLUDE_FILE" ;;
esac

: > "$root/meta/extra_files.txt"
if [ -f "$include_path" ]; then
  while IFS= read -r spec; do
    # Strip comments + trim whitespace
    spec="${spec%%#*}"
    spec="$(printf "%s" "$spec" | sed -e 's/^[[:space:]]*//' -e 's/[[:space:]]*$//')"
    [ -z "$spec" ] && continue

    # safety: disallow absolute and parent traversal
    case "$spec" in
      /*|*..*) echo "WARN: skip unsafe include spec: $spec" >> "$root/meta/warnings.txt"; continue ;;
    esac

    # If it looks like a glob, use git pathspec magic :(glob)
    if printf "%s" "$spec" | grep -Eq '[*?\[]'; then
      git -C "$toplevel" ls-files -- ":(glob)$spec" 2>/dev/null >> "$root/meta/extra_files.txt" || true
    else
      # Try tracked path first
      git -C "$toplevel" ls-files -- "$spec" 2>/dev/null >> "$root/meta/extra_files.txt" || true
      # Also allow untracked explicit file path
      if [ -f "$toplevel/$spec" ]; then
        echo "$spec" >> "$root/meta/extra_files.txt"
      fi
    fi
  done < "$include_path"
fi
sort -u "$root/meta/extra_files.txt" -o "$root/meta/extra_files.txt"

if [ -s "$root/meta/extra_files.txt" ]; then
  while IFS= read -r f; do
    [ -z "$f" ] && continue
    [ -f "$toplevel/$f" ] || continue
    mkdir -p "$root/files/$(dirname "$f")"
    cp "$toplevel/$f" "$root/files/$f"
  done < "$root/meta/extra_files.txt"
fi

# --- EVIDENCE ---
if [ -n "${EVIDENCE_FILE:-}" ]; then
  case "$EVIDENCE_FILE" in
    /*) ev_path="$EVIDENCE_FILE" ;;
    *)  ev_path="$toplevel/$EVIDENCE_FILE" ;;
  esac

  if [ -f "$ev_path" ]; then
    cp "$ev_path" "$root/evidence/$(basename "$ev_path")"
  else
    echo "WARN: EVIDENCE_FILE specified but not found: $EVIDENCE_FILE" >> "$root/meta/warnings.txt"
  fi
else
  ev=$(
    ls -1t "$toplevel/.local/prverify"/prverify_*_"${anchor7}".md 2>/dev/null | head -n 1 \
    || ls -1t "$toplevel/.local/prverify"/prverify_*_"${head12}".md 2>/dev/null | head -n 1 \
    || ls -1t "$toplevel/.local/prverify"/prverify_*_"${head7}".md 2>/dev/null | head -n 1 \
    || true
  )

  if [ -z "${ev:-}" ]; then
    ev="$(ls -1t "$toplevel/.local/prverify"/prverify_*.md 2>/dev/null | head -n 1 || true)"
    if [ -n "${ev:-}" ]; then
      echo "WARN: No prverify log found for ANCHOR (${anchor7}) or HEAD (${head12}); included latest: $(basename "$ev")" >> "$root/meta/warnings.txt"
    fi
  fi

  if [ -n "${ev:-}" ] && [ -f "$ev" ]; then
    cp "$ev" "$root/evidence/$(basename "$ev")"
  else
    echo "WARN: No evidence log found in .local/prverify" >> "$root/meta/warnings.txt"
  fi
fi

# --- INDEX ---
{
  echo "# Review Bundle"
  echo
  echo "project: $project"
  echo "mode:    $MODE"
  echo "base:    $(cat "$root/meta/base_sha.txt")"
  echo "head:    $(cat "$root/meta/head_sha.txt")"
  echo "branch:  $(cat "$root/meta/branch.txt")"
  echo
  echo "## Quick Start"
  echo "1) meta/changed_files.txt"
  echo "2) files/ (latest snapshots)"
  echo "3) patch/series.patch"
  if [ "$MODE" = "wip" ]; then
    echo "4) patch/wip_index.patch + patch/wip_worktree.patch"
    echo "5) evidence/ (if present)"
  else
    echo "4) evidence/ (if present)"
  fi
  echo
  echo "## Optional"
  echo "- meta/extra_files.txt (resolved from $INCLUDE_FILE if exists)"
  echo "- meta/warnings.txt (always present; check if evidence missing)"
} > "$root/INDEX.md"

# --- PACK ---
tar_opts=()
if tar --help 2>/dev/null | grep -q -- '--no-xattrs'; then tar_opts+=(--no-xattrs); fi
if tar --help 2>/dev/null | grep -q -- '--no-mac-metadata'; then tar_opts+=(--no-mac-metadata); fi

COPYFILE_DISABLE=1 tar ${tar_opts[@]+"${tar_opts[@]}"} -czf "$out" -C "$tmp" review

ls -lh "$out"
echo "OK: $out"
