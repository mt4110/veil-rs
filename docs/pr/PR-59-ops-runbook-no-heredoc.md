# PR59 — Ops Runbook: No-Heredoc / StrictMode-safe manual CLI patterns

## Context
PR58で露呈した運用地雷を、runbook上の “型” として固定する。
- heredoc待ち（入力待ちで固まる）
- `set -u` + プロンプト由来（例：`__git_ps1`）などの未初期化変数事故
- ターミナル跨ぎで `REPO/EVDIR/ISSUE` がズレる事故

## Objective
「低性能モデル + 手動」でも事故らず、closeout / evidence 収集を毎回同じ手順で回せる状態にする。

## Scope (In)
- `docs/runbook/always-run.md` に “No-Heredoc / StrictMode-safe” の手動CLIパターンを追記
- plan/task を Always Run 形式で追加（停止条件 / 分岐 / skip理由を明文化）

## Scope (Out)
- 実装コード変更
- CI挙動の変更

## Definition of Done
- runbook だけ読めば、手動でも closeout/evidence が事故らず完遂できる
- 例コマンドが heredoc を使っていない
- `set -u` の推奨が host 側に漏れていない（subshell 内に封印されている）
- `nix run .#prverify` PASS
- prverify evidence が `docs/evidence/prverify/` に永続化されている

## Always Run Contract
- SOT: `docs/pr/PR-59-ops-runbook-no-heredoc.md`
- plan/task:
  - `docs/pr/PR-59-ops-runbook-no-heredoc/plan.md`
  - `docs/pr/PR-59-ops-runbook-no-heredoc/task.md`
- **Latest prverify report:** `docs/evidence/prverify/`
