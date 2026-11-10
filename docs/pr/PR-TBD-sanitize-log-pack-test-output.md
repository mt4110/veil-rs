# ログ RulePack のテスト失敗時に出力内容を表示しない

Status: `archive`

## SOT

- 対象は `feat/sanitize-log-pack-test-output`。ベースは `main` の `c2e5330`。

## What

- Code Scanning アラート #1 が指摘した統合テストで、失敗時のファイル抜粋と子プロセスの出力を固定メッセージに置き換えた。プレースホルダーとマスク結果の検証は維持した。
- 本番 CLI の RulePack 読み込みエラーは別件として扱う。

## Verification

- `cargo test --locked --offline -p veil-cli --test init_log_pack_test` は7件すべて成功した。
- `cargo fmt --all -- --check` と `git diff --check` が通過した。

## Evidence

- [対象テスト](../../crates/veil-cli/tests/init_log_pack_test.rs)
