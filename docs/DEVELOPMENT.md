# Developer Guide

これは `veil-rs` の開発者およびコントリビューター向けのガイドです。
利用者向けのマニュアルは [README.md](../README.md) を参照してください。

## Minimum Supported Rust Version (MSRV)
We support the latest stable Rust and **MSRV 1.92.0**.
- **Patch Policy**: Patch releases never bump MSRV.
- **Minor Policy**: Minor releases may bump MSRV (documented in release notes).

## Build from Source (ソースコードからビルド)

開発に参加する場合や、最新の `main` ブランチを試したい場合の手順です。

```bash
git clone https://github.com/mt4110/veil-rs.git
cd veil-rs
```

### Nix (開発・検証用)

`nix develop` は固定された開発・検証環境です。通常CLIではGitのHTTPS/SSHとOpenSSL依存を外しました。
macOSの配布候補は標準SDKの環境でビルドし、NixやHomebrewへの動的リンクが残っていないことを検査します。[CI・リリース条件](ci/oss-cli-release.md) を参照してください。

```bash
# 1. 開発環境に入る
nix develop

# 2. ローカルでビルド
cargo build --locked --release

# またはシステムにインストール (veilコマンドを更新)
cargo install --locked --path ./crates/veil-cli
```

> [!TIP]
> **Check MSRV (1.92.0)**
> ```bash
> nix develop .#msrv
> ```

### Nixを使わない場合
Rust (1.92.0以上) がインストールされている必要があります。

```bash
cargo build --locked --release
```

バイナリは `./target/release/veil` に生成されます。

## Testing

veil-rs includes tests for secret detection rules (Slack, AWS, GitHub PATs, etc.).

To avoid GitHub Push Protection blocking pushes, new tests should generate fake tokens at runtime rather than embedding complete secret-looking literals.
Some legacy fixtures still contain fake literals; their exclusion depends on the effective scan configuration.

See [docs/TESTING_SECRETS.md](TESTING_SECRETS.md) for the full “Safety Contract”
and guidelines on adding new secret tests.

## Local OSS profile

Default `cargo build` / `cargo test` targets config, core, and CLI.
The existing Evidence Pack v1 verifier is part of default and minimal CLI checks; no governance feature is required.
Legacy service/UI/Guardian members require explicit package selection or `--workspace`.
Run `python3 scripts/check_architecture.py` before changing crate dependencies.

Release candidates are checked with `python3 scripts/check_cli_release.py --binary <binary>`.

## OSS CLIへの貢献

検知・ルール・マスキングは `veil-core`、引数・表示・終了条件は `veil-cli`、設定型と検証は `veil-config` に置きます。
通常の依存グラフへネットワーククライアント・ライセンス認証・サービスを追加せず、旧機能は明示したpackage/featureで分離します。[アーキテクチャ](architecture/local-oss.md)を参照してください。

変更時は発生条件、期待する結果、失敗時の振る舞い、確認方法を記載します。本物の鍵を使わず、架空の値を実行時に生成してください。
検知・マスキング・baseline・終了コードを変える場合は、異常系や境界を含む回帰確認を追加します。
依存追加は必要性・ライセンス・セキュリティへの影響を確認します。生成したレポートとbaselineのメタデータも公開前に確認してください。

リポジトリのリリース検証にはPython 3.11以上が必要です。ソースのルートで実行します。

```bash
cargo fmt --all -- --check
cargo clippy --locked -p veil-cli -p veil-core -p veil-config --all-targets -- -D warnings
cargo test --locked
python3 scripts/check_docs_taxonomy.py
python3 scripts/check_architecture.py
python3 -B -m unittest discover -s scripts -p 'test_check_*.py'
```

feature境界に関係する変更では最小構成とJP機能・独立LSPも確認します。手順と対象OSは [リリース条件](ci/oss-cli-release.md)にまとめています。
チュートリアルを変更したら実CLIで手順と非ゼロ終了を確認し、失敗を成功へ変換しないでください。
リリース候補は既存作業とprivateファイルから公開する差分を分離し、PRごとのSOT・検証記録を付けます。このガイドは公開・マージの承認を代替しません。
