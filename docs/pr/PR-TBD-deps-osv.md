# 依存関係とローカル監査の更新

Status: `archive`

Scope: PR の変更とローカル検証の記録

## SOT

- 対象ブランチは `feat/deps-osv`。ベースは `main` の `b0af4e5`。
- Rust 1.92.0 の最低対応と CLI の既存動作を維持しながら、依存と配布用のライセンス台帳を更新する。

## What

- Cargo、npm、Nix の依存とロックを更新し、API の変更に伴う呼び出し箇所と OpenAPI 生成物を調整した。SQLx は Rust 1.92.0 対応の 0.8.6、TypeScript は `svelte-check` が対応する 6.0.3 を使用する。
- 既定 CLI の依存グラフに合わせて、第三者ライセンス台帳と原文を更新した。
- Scorecard 5.5.0 への nixpkgs 更新で据え置かれた Darwin 用の依存ハッシュを、Apple Silicon に限り補正した。公式タグのソースから Go モジュールを取得し、公開チェックサムDBと `go mod verify` で検証した結果を使った。
- pre-commit の OSV-Scanner を、ロック変更の有無にかかわらず実行する。テスト用フィクスチャと生成・作業ディレクトリを対象から除いた。

## Verification

- Rust 1.92.0 の `cargo build --locked --workspace --all-features` と `cargo test --locked --workspace --all-features --no-fail-fast` が成功。通常の528テストに加え、無視指定の大規模テンプレートテストを個別に実行した。最小構成の CLI テスト、Clippy、`cargo fmt --all -- --check` も成功。
- Go 1.27.3 の `go build ./...` と `go test ./...`、フロントエンドの `npm ci --ignore-scripts`、`npm run check`、`npm run build`、OpenAPI 生成物チェックが成功。
- `nix flake check --all-systems --no-build` が成功。Apple Silicon 上で `veil`、`reviewbundle`、`cockpit` を実ビルドし、`cockpit` が参照する Scorecard 5.5.0 の起動を確認した。
- Apple Silicon 向けのローカル配布 ZIP で `check_cli_release.py`、`verify_cli_archive.py`、`check_staged_workflow.py`（18ケース）、`check_cli_install.py`（21ケース）が成功。配布関連を含む Python テスト119件も成功。
- Cargo Audit は0件。OSV-Scanner は README だけを指定した実行でも起動し、npm 開発依存 `deepmerge@4.3.1` の [GHSA-gcqx-w48m-w2h4](https://osv.dev/GHSA-gcqx-w48m-w2h4) を1件報告して終了コード1を返す。修正版は確認できていない。

## Evidence

- [第三者ライセンス台帳](../../third_party/manifest.json)
- [nixpkgs の Scorecard 5.5.0 更新差分](https://github.com/NixOS/nixpkgs/commit/7eb3ef0b764cdd5366fd1d3ad799ccd03bbb11408)
- [OSV のフック定義](../../.pre-commit-config.yaml)

## Non-goals

- 実環境の Google OAuth、Linux musl・Windows の配布物はローカル検証の対象外。
- Git の共有フック設定と配布済み成果物は対象外。

## Rollback

- 依存、互換性修正、ライセンス台帳の整合を保つため、PR を一単位で戻す。
