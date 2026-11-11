# v0.22.2 公開後の導入案内

## SOT

日英の README と導入ガイドから v0.22.2 の ZIP を取得し、展開前に検証できるようにする。

## What

- macOS arm64 と Linux x86_64 musl の取得先、ZIP 名、CLI の版表示を v0.22.2 に揃える。
- 選んだ ZIP の SHA256SUMS、Ed25519 分離署名、SLSA 来歴証明を検証する。来歴証明の検証ではビルドワークフローと配布元コミットを指定する。

## Verification

- 公開された ZIP 2本を再取得し、それぞれの SHA256SUMS、分離署名、SLSA 来歴証明を検証した。
- macOS arm64 で公開済み CLI と更新後の文書を使った確認用 ZIP の配布チェック、展開後の実行、インストーラーの21テスト、日英の初回スキャンとステージ点検が通った。Windows は保留。

## Evidence

- [v0.22.2 Release](https://github.com/mt4110/veil-rs/releases/tag/v0.22.2)
- [配布元コミットと来歴証明の生成結果](https://github.com/mt4110/veil-rs/actions/runs/42972013000)
