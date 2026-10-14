#!/usr/bin/env bash
set -euo pipefail
script_dir="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
hooks_dir="$script_dir/../.githooks"
hooks_dir="$(CDPATH= cd -- "$hooks_dir" && pwd)"
# An absolute path also protects older sibling worktrees without editing their files.
# Existing hooks are retained on disk; the bundled formatter guard is still called.
git config --local core.hooksPath "$hooks_dir"
printf 'Enabled Git hooks for this repository and its worktrees: %s\n' "$hooks_dir"
printf 'SOT base: %s\n' "$(git config --get sot.baseRef || printf 'origin/main')"
