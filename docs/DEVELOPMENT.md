# Developer Guide

これは `veil-rs` の開発者およびコントリビューター向けのガイドです。
利用者向けのマニュアルは [README.md](../README.md) を参照してください。

## Minimum Supported Rust Version (MSRV)
We support the latest stable Rust and **MSRV 1.82.0**.
- **Patch Policy**: Patch releases never bump MSRV.
- **Minor Policy**: Minor releases may bump MSRV (documented in release notes).

## Build from Source (ソースコードからビルド)

開発に参加する場合や、最新の `main` ブランチを試したい場合の手順です。

```bash
git clone https://github.com/mt4110/veil-rs.git
cd veil-rs
```

### Nix (推奨・Apple Siliconでは必須)

本プロジェクトは `nix develop` 環境での開発を前提としています。
特に macOS (Apple Silicon) では、OpenSSL などのシステムライブラリのリンクエラーを防ぐため、**必ずこの環境内で作業してください**。

```bash
# 1. 開発環境に入る (必須)
nix develop

# 2. ローカルでビルド
cargo build --release

# またはシステムにインストール (veilコマンドを更新)
cargo install --path ./crates/veil-cli
```

> [!TIP]
> **Check MSRV (1.82.0)**
> ```bash
> nix develop .#msrv
> ```

### Nixを使わない場合
Rust (1.82.0以上) がインストールされている必要があります。

```bash
cargo build --release
```

バイナリは `./target/release/veil` に生成されます。

## Testing

veil-rs includes tests for secret detection rules (Slack, AWS, GitHub PATs, etc.).

To avoid GitHub Push Protection blocking pushes, we **never** hard-code real-looking secrets
as string literals. Instead, tests generate fake tokens at runtime via helper functions.

See [docs/TESTING_SECRETS.md](TESTING_SECRETS.md) for the full “Safety Contract”
and guidelines on adding new secret tests.
