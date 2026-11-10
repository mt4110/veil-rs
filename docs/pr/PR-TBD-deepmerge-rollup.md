# veil-pro のフロントエンドから deepmerge を除去

Status: `archive`

Scope: フロントエンドのビルド依存とローカル配布検証の記録

## SOT

- 対象ブランチは `feat/deepmerge-rollup-evaluation`。ベースは `main` の `df8ecb3`。
- [Dependabot アラート #83](https://github.com/mt4110/veil-rs/security/dependabot/83) の依存経路を、既存の Vite・Svelte・TypeScript を維持して外す。

## What

- `@sveltejs/vite-plugin-svelte` を `rollup-plugin-svelte@7.2.3` に置き換える。ビルドでは CSS を別ファイルに出力し、開発時はコンポーネントから注入する。
- 開発時の Svelte ファイル編集はページ全体の再読み込みになる。この挙動は採用条件として許容する。

## Verification

- `npm ci` 後、ロックファイルと `npm ls --all --json` の両方に `deepmerge` がないことを確認した。
- `npm run check` はエラー・警告ともにゼロ、`npm run build` は成功、`npm audit` は検出ゼロ。
- 開発サーバーで主要な Svelte ファイルの配信と、編集時の `full-reload` イベントを確認した。
- `cargo check --locked --offline -p veil-pro`、CLI 配布ガード、Python テスト119件が通過した。
- Rust 1.92.0 で作成した macOS arm64 の CLI 配布 ZIP で、展開後の第三者表記とチェックサム、ステージ済みファイルの検査、インストール、日英オンボーディングを確認した。

## Evidence

- [フロントエンドのロックファイル](../../crates/veil-pro/frontend/package-lock.json)
- [Vite の設定](../../crates/veil-pro/frontend/vite.config.ts)
- [CLI 配布検証手順](../ci/oss-cli-release.md)
