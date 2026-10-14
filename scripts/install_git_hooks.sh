#!/usr/bin/env bash
set -euo pipefail
script_dir="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
hooks_dir="$(CDPATH= cd -- "$script_dir/../.githooks" && pwd)"
current_hooks="$(git rev-parse --path-format=absolute --git-path hooks)"

# core.hooksPath replaces Git's entire lookup directory, not just pre-commit.
# Refuse to disable any independently configured hook manager, even if empty.
if configured_hooks="$(git config --get core.hooksPath)"; then
  current_canonical=""
  if [ -d "$current_hooks" ]; then
    current_canonical="$(CDPATH= cd -- "$current_hooks" && pwd)"
  fi
  if [ "$current_canonical" != "$hooks_dir" ]; then
    printf 'Refusing to replace existing core.hooksPath: %s\n' "$configured_hooks" >&2
    printf 'Keep the existing hooks and integrate scripts/check_pr_sot.py --staged into their pre-commit flow.\n' >&2
    exit 1
  fi
else
  # Default .git/hooks may contain real hooks as well as Git's inactive samples.
  for hook in "$current_hooks"/*; do
    case "$hook" in *.sample) continue ;; esac
    if [ -f "$hook" ] && [ -x "$hook" ]; then
      printf 'Refusing to disable existing executable hook: %s\n' "$hook" >&2
      printf 'Integrate the SOT check into the existing hooks; configuration was not changed.\n' >&2
      exit 1
    fi
  done
fi

# An absolute path also protects sibling worktrees without source/index edits.
git config --local core.hooksPath "$hooks_dir"
printf 'Enabled Git hooks for this repository and its worktrees: %s\n' "$hooks_dir"
printf 'SOT base: %s\n' "$(git config --get sot.baseRef || printf 'origin/main')"
