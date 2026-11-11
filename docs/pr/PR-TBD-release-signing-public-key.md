# CLI 配布 ZIP の署名検証用公開鍵

## SOT

- 対象は `feat/release-signing-public-key`。ベースは `main` の `2f12363`。

## What

- リリース専用 Ed25519 公開鍵を追加し、配布ガイドに SHA-256 フィンガープリントと、確認済みのコミットから鍵を取得する手順を記載する。

## Verification

- 配布ガイドのコマンドで `2f12363` の macOS arm64 と Linux x86_64 musl の候補 ZIP を検証した。両方の署名は有効で、改変した各 ZIP と誤ったフィンガープリントは拒否された。
- CLI 配布ガードと関連テスト24件、文書分類、`git diff --check` が通過した。

## Evidence

- [Ed25519 公開鍵](../security/veil-rs-release-ed25519.pub)
- [配布ガイド](../ci/oss-cli-release.md)
- [候補 ZIP のビルドと来歴証明の検証](https://github.com/mt4110/veil-rs/actions/runs/42957812093)
