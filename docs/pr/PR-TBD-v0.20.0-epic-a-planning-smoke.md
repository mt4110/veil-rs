# PR-TBD-v0.20.0-epic-a-planning-smoke: Deterministic P0 CLI smoke suite (trycmd)

## Why (Background)
- CLIの“契約”を先に固めるため（機能を増やす前に、壊れ方を決定論に寄せる）。
- CIでの揺れ（ANSI/時刻/環境差）を吸収し、変更が入った瞬間にテストが確実に叫ぶ状態にしたい。

## Summary
- trycmdベースのP0 Smoke Suiteを追加し、主要コマンドの振る舞いをスナップショットで固定。
- `--no-color` や tolerant matcher（`[..]` 等）で、環境差でのフレークを減らす。

## Changes
- [x] P0 CLI Smoke Suite（trycmd）の追加/更新
- [x] fixtures（safe/secret/log/help/config等）の整備
- [x] 決定論 hardening（色・揺れる値の吸収）

## Non-goals (Not changed)
- [ ] `veil init (minimal)` の実装
- [ ] `veil guardian (offline-safe)` の契約化（cache無し安全終了）
- [ ] ネット必須の精密チェック（別レーンに分離予定）

## Impact / Scope
- CLI: 出力の決定論性を強化（色/揺れ対策）
- CI: smoke suite を常時実行し、回帰を即検出
- Docs: PR SOT を追加（このファイル）
- Rules: 変更なし
- Tests: trycmd smoke + 既存テスト

## Verification

### Commands
```bash
cargo test -p veil-cli --test cli_tests
# optional
cargo test --workspace

