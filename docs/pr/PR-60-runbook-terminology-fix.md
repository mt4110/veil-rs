# PR60 — Runbook Terminology Fix: “bash -lc is a separate bash process (not a subshell)”

## Objective
PR59 runbook の表現を技術的に正確化し、`bash -lc` を “subshell” ではなく “別 bash プロセス” として統一する。
隔離の意図（シェル状態の隔離）と用語の厳密さが混同されない状態にする。

## Changes
- `docs/runbook/always-run.md`: `bash -lc` に関する記述を “subshell” → “別 bash プロセス” に置換
- `docs/runbook/always-run.md`: Terminology Control（用語定義）を追記（別 bash プロセス vs subshell（厳密））

## Verification (Always Run Contract)
### 1) Terminology audit
- Command: `rg -n "subshell" docs/runbook/always-run.md`
- Result: PASS（用語定義セクション以外で subshell を使わない）

### 2) prverify
- Command: `nix run .#prverify`
- Result: PASS
- Report: `docs/evidence/prverify/prverify_20260213T000902Z_26bbf7e.md`

### 3) Cockpit Check
- Command: `nix run .#check`
- Result: PASS

## Always Run Contract
- SOT: `docs/pr/PR-60-runbook-terminology-fix.md`
- plan/task:
  - `docs/pr/PR-60-runbook-terminology-fix/plan.md`
  - `docs/pr/PR-60-runbook-terminology-fix/task.md`
- Latest prverify report: `docs/evidence/prverify/prverify_20260213T000902Z_26bbf7e.md`
