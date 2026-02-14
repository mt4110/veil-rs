# S10-06 Plan — Post-merge Copilot PR Audit & Remediation (main hardening)

goal:
- Copilotがmainへマージした複数PR（~5本）について、差分と影響範囲を監査し
  - (A) 破綻/ノイズ（絶対パス、壊れたMarkdown、CWD依存など）を修正
  - (B) 将来同型事故を再発させないガード（CI/ドキュメント/運用手順）を追加
- すべてを「証拠→修正→証拠」で閉じる（嘘を付かない）

constraints:
- main の verify-only PASS を破壊しない（壊れていたら原因特定してから補修）
- 再現性：同じ入力（git sha / PR一覧）で同じ結論になる形（ログ・一覧を保存）
- 絶対パスを docs に残さない（portableの敵）
- runbook/ops は “監査ログ” として未来に読まれる前提で書く

inputs:
- git history (main)
- merged PR list (GitHub CLI)
- changed files list per PR
- local verification (prverify 等)

plan:

0. establish base
- base_sha := current origin/main HEAD
- if working tree dirty:
  - error "dirty worktree => STOP (stash/commit first)"
- else:
  - continue

1. enumerate suspect PRs (copilot merges)
- merged_prs := list merged PRs on base=main (recent 30)
- suspect := filter where author/login contains "copilot" OR merged_by contains "copilot" OR title indicates bot merge
- if suspect is empty:
  - skip "no copilot-merged PR found in recent window"
- else:
  - continue

2. compute impact per PR (deterministic)
- for each pr in suspect:
  - files := list changed files for pr
  - if files contains code (Go/Rust/Shell):
    - mark pr as "code-impact"
  - else:
    - mark pr as "docs-only"
  - store: pr number, merge commit sha, files list (sorted)

3. run safety checks on changed files
- checks:
  - markdown_fence_integrity: count code-fence markers (three backticks) per file; if odd => broken
  - absolute_path_leak: search "<HOME>/", "<DRIVE>:\" in docs/**/*.md and evidence logs
  - cwd_dependency: search exec.Command("bash", ...) without cmd.Dir pinned to repo root
  - tmp_files: detect accidental tracked files like pr_body.txt / scratch / .local paths
- for each check:
  - if hit severity == high (e.g., broken markdown fences in docs shipped):
    - mark remediation required
  - else:
    - continue

4. verification on current main (baseline)
- run prverify (or nearest equivalent)
- if fail:
  - error "main verification FAIL => STOP (must fix before adding guardrails)"
- else:
  - record PASS evidence and continue

5. remediation branch
- create branch s10-06-post-merge-copilot-audit-v1
- apply minimal fixes:
  - docs: remove absolute paths; fix code fences; ensure evidence is portable
  - code: pin cmd.Dir to repo root where shell scripts executed
  - remove accidental tracked temp files
- after each fix group:
  - rerun verification
  - append evidence log

6. prevention / guardrails
- if repo already has SOT requirement in CI:
  - ensure docs/pr template exists and runbook explains "how to add SOT"
- else:
  - add lightweight CI guard (or docs-only guard) consistent with current pipeline
- add docs/ops note: "copilot PR merge policy" (what to verify before merge)

7. finalize
- prepare SOT for this remediation PR under docs/pr/
- attach evidence paths
- create PR with SOT/証拠スタイル（ガチガチ版）
- stop conditions:
  - if any verification step fails => error and stop (no pretending)
  - skip requires 1-line reason
