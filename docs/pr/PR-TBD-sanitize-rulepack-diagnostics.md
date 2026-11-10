# RulePack の TOML 解析エラーから原文を除く

Status: `archive`

## SOT

- 対象は `feat/sanitize-rulepack-diagnostics`。ベースは `main` の `e1aec95`。

## What

- マニフェストとルールファイルの TOML 解析エラーから原文付きのパーサー診断を除く。呼び出し元のファイル名と、取得できる場合の行・列を残す。
- 合成データで CLI の標準エラーと終了コードを確認する回帰テストを追加する。

## Verification

- 修正前は不正なマニフェストとルールファイルの両方で、合成マーカーを含む原文が標準エラーに出た。
- 修正後は両ファイルで、標準エラーにファイル名と行・列が残り、合成マーカーは出力されず、終了コードは2だった。CLI 統合テストは8件成功、RulePack コアテストは13件成功・既存の1件は ignore。
- `cargo fmt --all -- --check` と `git diff --check` が通過した。

## Evidence

- [RulePack の解析処理](../../crates/veil-core/src/rules/pack.rs)
- [CLI 統合テスト](../../crates/veil-cli/tests/init_log_pack_test.rs)
