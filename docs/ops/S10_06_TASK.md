# S10-06 Task — Post-merge Copilot PR Audit & Remediation (main hardening)

## 0) Base snapshot (main)

- [x] cd "$(git rev-parse --show-toplevel)"
- [x] git switch main
- [x] git fetch origin --prune
- [x] git pull --ff-only
- [x] git status -sb
- [x] if `git status --porcelain=v1` is not empty:
  - [x] error "dirty worktree => STOP"
- [x] BASE_SHA="$(git rev-parse HEAD)"
- [x] echo "BASE_SHA=$BASE_SHA"

## 1) Enumerate merged PRs (recent) and filter copilot

- [x] gh pr list --base main --state merged --limit 30 --json number,title,author,mergedAt,mergeCommit --jq '.[] | "\(.number)\t\(.author.login)\t\(.mergeCommit.oid)\t\(.mergedAt)\t\(.title)"' | tee ".local/copilot_audit_recent_prs.tsv"
- [x] if ".local/copilot_audit_recent_prs.tsv" not created:
  - [x] error "failed to list PRs => STOP"
- [x] rg -n "copilot" -i ".local/copilot_audit_recent_prs.tsv" || true
- [x] if no line matches "copilot":
  - [x] skip "no copilot author/login in recent merged PRs"
  - [x] (OPTIONAL) continue with manual suspect list if user has PR numbers
- [x] else:
  - [x] continue

## 2) Collect changed files per suspect PR (sorted)

- [x] mkdir -p .local/copilot_audit
- [x] for PR in (suspect PR numbers from the TSV):
  - [x] gh pr diff "$PR" --name-only | sort > ".local/copilot_audit/pr_${PR}_files.txt"
  - [x] if file list empty:
    - [x] skip "PR ${PR}: no files returned"
    - [x] continue
  - [x] else:
    - [x] continue
- [x] if no ".local/copilot_audit/pr_*_files.txt" exists:
  - [x] error "no PR file lists collected => STOP"

## 3) Static risk scans (portable / deterministic)

### 3-A) Absolute path leak in docs

- [x] rg -n --hidden -S '(/Users/|/home/|[A-Za-z]:\\)' docs -g'*.md' | tee ".local/copilot_audit/abs_path_hits.txt" || true
- [x] if ".local/copilot_audit/abs_path_hits.txt" is non-empty:
  - [x] mark "remediation required: absolute paths in docs"
- [x] else:
  - [x] skip "no absolute path leak found"

### 3-B) Broken markdown fences in touched docs

- [x] python - <<'PY'
import pathlib, sys
paths = []
for p in pathlib.Path(".local/copilot_audit").glob("pr_*_files.txt"):
    for line in p.read_text().splitlines():
        if line.startswith("docs/") and line.endswith(".md"):
            paths.append(line)
paths = sorted(set(paths))
bad = []
for f in paths:
    try:
        txt = pathlib.Path(f).read_text(encoding="utf-8", errors="replace")
    except FileNotFoundError:
        continue
    if txt.count("```") % 2 == 1:
        bad.append(f)
out = pathlib.Path(".local/copilot_audit/broken_fences.txt")
out.write_text("\n".join(bad) + ("\n" if bad else ""), encoding="utf-8")
print(f"checked={len(paths)} bad={len(bad)} -> {out}")
PY
- [x] if ".local/copilot_audit/broken_fences.txt" is non-empty:
  - [x] mark "remediation required: broken markdown fences"
- [x] else:
  - [x] skip "no broken fences in touched docs"

### 3-C) CWD dependency for shell execution in Go

- [x] rg -n 'exec.Command("bash"|exec.Command("sh"' internal cmd -S | tee ".local/copilot_audit/exec_bash_hits.txt" || true
- [x] rg -n '.Dir\s*=' internal cmd -S | tee ".local/copilot_audit/cmd_dir_hits.txt" || true
- [x] if exec hits exist AND no corresponding cmd.Dir pinning near them:
  - [x] mark "remediation required: cmd.Dir not pinned to repo root"
- [x] else:
  - [x] skip "cmd.Dir pinning seems present (manual review still recommended)"

### 3-D) Accidental tracked temp files

- [x] rg -n -S 'pr_body.txt|TODO(TEMP)|scratch' -S . | tee ".local/copilot_audit/tmp_suspects.txt" || true

## 4) Baseline verification on main

- [x] command -v nix >/dev/null 2>&1 && nix run .#prverify || true
- [x] if verification FAIL:
  - [x] error "main verification FAIL => STOP (fix before guardrails)"

## 5) Remediation branch

- [x] BR="s10-06-post-merge-copilot-audit-v1"
- [x] git switch -c "$BR"
- [x] git status -sb

## 6) Apply fixes (minimal diffs)

### 6-A) Fix docs absolute paths (portable)

- [x] if ".local/copilot_audit/abs_path_hits.txt" empty:
  - [x] skip "no abs path fixes needed"
- [x] else:
  - [x] for each hit line:
    - [x] edit target md to replace absolute path with repo-relative path or basename
    - [x] if replacement unclear:
      - [x] error "cannot safely rewrite abs path => STOP (needs decision)"

### 6-B) Fix broken markdown fences

- [x] if ".local/copilot_audit/broken_fences.txt" empty:
  - [x] skip "no fence fixes needed"
- [x] else:
  - [x] for each md in broken_fences:
    - [x] open file and remove extra ``` or add missing closing fence
    - [x] re-run fence checker for that file
    - [x] if still odd:
      - [x] error "fence still broken => STOP"

### 6-C) Pin script execution to repo root (cmd.Dir)

- [x] if "remediation required: cmd.Dir not pinned" not marked:
  - [x] skip "no cmd.Dir fix needed"
- [x] else:
  - [x] implement repoRoot := `git rev-parse --show-toplevel` (Go helper)
  - [x] set cmd.Dir = repoRoot for review_bundle execution
  - [x] ensure evidence records relative paths only (portable)

### 6-D) Remove accidental tracked temp files

- [x] if tmp suspects include tracked files:
  - [x] git rm -f <tracked-temp-file>
  - [x] else:
    - [x] skip "no tracked temp files"

## 7) Verification after fixes

- [x] command -v nix >/dev/null 2>&1 && nix run .#prverify || true
- [x] if FAIL:
  - [x] error "post-fix verification FAIL => STOP"

## 8) Evidence & SOT for this remediation PR

- [x] append a new section to docs/ops/S10_evidence.md:
  - [x] list suspect PRs + merge SHAs (from TSV)
  - [x] list applied fixes (abs paths / fences / cmd.Dir / temp files)
  - [x] include prverify PASS report path
- [x] create SOT file under docs/pr/ (manual scaffold):
  - [x] docs/pr/PR-TBD-<release>-epic-<epic>-copilot-post-merge-audit.md
  - [x] include: SOT + evidence pointers + rollback
- [x] if SOT path naming unclear:
  - [x] error "SOT naming unclear => STOP (choose epic/release)"

## 9) Commit / push / PR

- [x] git status -sb
- [x] git add -A
- [x] git commit -m "chore(s10-06): post-merge copilot audit fixes + evidence"
- [x] git push -u origin "$BR"
- [x] gh pr create --base main --head "$BR"
- [ ] paste PR body in veil-rs SOT/証拠スタイル（ガチガチ版）
