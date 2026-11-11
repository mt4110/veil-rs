# RulePack TOML のファジング

## SOT

- 対象は `feat/rulepack-fuzzing`。ベースは `main` の `fd149ab`。

## What

- 既存の RulePack マニフェストとルールファイルのパーサーに、同じ UTF-8 入力を渡す。ルールファイル側は正規表現のコンパイル、バリデーターの解決、ID の重複検査も通る。入力をパスとして扱う処理は対象外。
- 独立した Cargo ワークスペースとロックファイルに fuzzing 用の依存を置く。ClusterFuzzLite は関連ファイルを変更した PR で AddressSanitizer を使って60秒間実行する。

## Verification

- macOS arm64 で合成シード3件から10,000入力を実行し、クラッシュはなかった。
- ClusterFuzzLite 用の Linux amd64 コンテナでターゲットのビルドとシード3件の梱包が成功した。手元の amd64 エミュレーションでは、ターゲットと最小の AddressSanitizer 付き C プログラムがともに終了コード137で停止した。ネイティブ Linux の GitHub Actions 上での実行は未確認。
- Rust 1.92.0 の RulePack テストは13件成功、失敗0件、既存の無視1件。
- Scorecard v5.6.0 のローカル検査は ClusterFuzzLite を検出し、Fuzzing、Pinned-Dependencies、Token-Permissions が各10/10。

## Evidence

- [ファジングターゲット](../../fuzz/fuzz_targets/rulepack_toml.rs)
- [ClusterFuzzLite ワークフロー](../../.github/workflows/cflite_pr.yml)

```sh
rustup run nightly cargo install cargo-fuzz --version 0.13.2 --locked --root "$PWD/.local/cargo-fuzz"
export PATH="$PWD/.local/cargo-fuzz/bin:$PATH"
mkdir -p .local/rulepack-fuzz-corpus
rustup run nightly cargo fuzz run --target-dir "$PWD/.local/fuzz-target" \
  rulepack_toml .local/rulepack-fuzz-corpus fuzz/seeds/rulepack_toml -- -runs=10000 -max_len=8192
```
