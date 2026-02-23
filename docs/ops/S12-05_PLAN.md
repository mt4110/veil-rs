# S12-05 PLAN: ci-repro runner/DI alignment (v1)

## Goal
Align ci-repro to prkit runner + DI conventions, as cleanup-only (no feature expansion).

## Non-Goals
- No new user-facing features unless required for alignment
- No behavior changes except wiring/structure (must be evidenced)

## Deliverables
- ci-repro entry aligns to prkit runner contract
- ci-repro core uses injected deps (FS/Exec/Env/Clock/Log etc.)
- Tests + docs + STATUS.md updated with evidence

## Path Discovery (must be real paths)
- Use rg to locate:
  - ci-repro implementation files
  - prkit runner interface / DI container (or deps struct)
- Record discovered paths in evidence logs

## Steps (stopless)
1. Baseline capture
   - Run `go test ./...`
   - Run `nix run .#prverify` (if available)
   - Run current ci-repro representative command(s)
   - Save logs under `.local/obs/s12-05_*/`
   - If any baseline fails: mark ERROR and stop further refactor (do not exit non-zero)

2. Runner alignment
   - Refactor CLI to call prkit runner (or wrap runner under existing CLI)
   - Ensure runner receives context + deps in prkit style
   - Keep behavior stable

3. DI alignment
   - Identify direct side-effect usage (`os/exec`, `os.Getenv`, `time`, filesystem)
   - Introduce deps interface/struct consistent with prkit patterns
   - Thread deps through runner -> core
   - Add minimal unit tests with fake deps where valuable

4. Evidence + Docs
   - Update docs describing how to run ci-repro via prkit runner
   - Update STATUS.md (S12-05 row + Last Updated + Evidence pointer)

## DoD / Acceptance
- `go test ./...` PASS
- `nix run .#prverify` PASS (or documented SKIP with reason if not available in env)
- ci-repro representative run(s) produce expected outputs
- STATUS.md updated and consistent

---

# Phase 2: Copilot Review Fixups

## Goal
Copilotレビュー2件を回収し、S12-05（ci-repro runner/DI整列）のPRを 最終マージ可能状態へ戻す。
「止まらない」＝「勝手に進めない」。STOPフラグで次工程をSKIPする。

## Non-Goal
- 仕様追加（レビュー指摘の解消に必要な最小変更のみ）
- 重い検証（cargo test全走など）を何回も回さない（最後に1回）

## Pseudocode
```python
STOP = 0
OBS = ".local/obs/s12-05_copilot_<UTC>"

try:
  repo_root = git rev-parse --show-toplevel
  if repo_root is empty:
    print ERROR; STOP=1

  if STOP==0:
    pr_number = (env PR) else gh pr view --json number
    if pr_number empty:
      print ERROR; STOP=1

  if STOP==0:
    # 1) Copilotレビュー採取（軽い）
    fetch review comments json -> OBS/reviews_*.json
    extract copilot-only summary -> OBS/copilot_summary.md
    if summary has 0 items:
      print SKIP (no copilot comments) ; STOP=1  # 嘘を付かない

  if STOP==0:
    # 2) 指摘を分類（死角の可視化）
    for each comment in copilot_summary:
      if type == "typo/docs":
        queue docs fixes
      else if type == "correctness/safety":
        queue code fixes (smallest)
      else if type == "determinism/DI/runner contract":
        queue contract fixes (smallest)
      else:
        queue "needs-human-judgement" with rationale
      continue

  if STOP==0:
    # 3) 実装修正（小さく、確実に）
    apply fixes one-by-one
    after each logical group:
      gofmt (targeted files only)
      run LIGHT tests (package-level)
      if LIGHT tests show failure:
        print ERROR; STOP=1  # 次へ進まない（止まらないが、進めない）

  if STOP==0:
    # 4) 重い検証は最後に1回（CPU事故防止）
    run nix run .#prverify  (single shot)
    parse output for PASS line
    if not PASS:
      print ERROR; STOP=1

  if STOP==0:
    # 5) 証拠更新
    update docs/pr/PR-<PR>-s12-05-*.md with:
      - head sha
      - prverify report path
      - reviewbundle strict filename + sha256
    update docs/ops/STATUS.md (S12-05: 99% Review + evidence pointer)

  print OK: phase=end
except Exception:
  print ERROR: unexpected (do not exit)
  print OK: phase=end
```
