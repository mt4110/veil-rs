# v0.18.0 PR#2 — CI workflow overwrite/skip UX (Epic B)

## Goal
- `veil init --ci github` 実行時に `.github/workflows/veil.yml` が既にある場合のUXを改善する

## Scope
- 既存ファイル検知
- デフォルト安全（上書きしない）
- 明示フラグで上書きできる
- テスト追加

## Acceptance Criteria
- 既存ファイルあり: デフォルトは skip（破壊しない）
- 明示指定で overwrite 可能（例: `--ci-overwrite`）
- non-interactive でも挙動が決定的（skip して案内）
- tests green

## Worklog
- 2026-01-12: SOT created
