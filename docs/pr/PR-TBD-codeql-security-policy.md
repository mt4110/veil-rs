# CodeQL とリポジトリの脆弱性報告窓口

Status: `archive`

## SOT

- 対象は `feat/codeql-security-policy`。ベースは `main` の `d93f81e`。

## What

- Rust と JavaScript/TypeScript を CodeQL で分析する。Actions はコミット SHA に固定し、書き込み権限は分析ジョブの `security-events` に限定する。
- リポジトリ直下の `SECURITY.md` に GitHub の非公開報告窓口と報告時に必要な情報を記載する。

## Verification

- `actionlint`、SOT ガード、CLI 配布ガードを通過した。
- OpenSSF Scorecard v5.6.0 のローカル比較では、SAST と Security-Policy が 0 から 10 に改善した。Token-Permissions と Pinned-Dependencies は 10 を維持した。
- GitHub 上での分析と SARIF アップロードは、ワークフローの初回実行後に確認する。

## Evidence

- [CodeQL ワークフロー](../../.github/workflows/codeql.yml)
- [Security Policy](../../SECURITY.md)
