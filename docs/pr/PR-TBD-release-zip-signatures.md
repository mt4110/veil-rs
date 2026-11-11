# CLI 配布 ZIP の分離署名手順

Status: `archive`

## SOT

- 対象ブランチは `feat/release-zip-signatures`。ベースは `main` の `5580d60`。

## What

- macOS arm64 と Linux x86_64 musl の最終 ZIP 2本に OpenSSH Ed25519 の分離署名を付け、信頼済み公開鍵で検証する手順を追加した。

## Verification

- 公開済み `v0.22.1` の ZIP のローカルコピーを `SHA256SUMS` と照合し、テスト鍵で署名した。元の ZIP 2本は署名検証に成功し、それぞれを改変したコピーは署名検証に失敗した。
- CLI 配布ガード、文書分類、`git diff --check` が通過した。

## Evidence

- [署名・検証手順](../ci/oss-cli-release.md#cli-配布-zip-の分離署名)
