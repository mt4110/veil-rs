# repro plan (local)

目的：**「検証して、証拠を書いて、戻し方も書く」** を *手順書の腐敗なし* で回す。

このリポジトリでは、再現手順を「CLIフラグ列」で固定すると壊れやすいです。
（将来フラグが増減して、未来の自分がコケる）

なので、**テスト駆動 + 設定駆動** を前提に寄せます。

## TL;DR（ワンコマンド）

```bash
nix run .#prverify
```

- smoke（trycmd） + workspace テスト
- 実行ログを `.local/prverify/` に Markdown として保存

詳細は `docs/ci/prverify.md` を参照。

## Manual fallback（Nixが使えない時）

```bash
cargo test -p veil-cli --test cli_tests
cargo test --workspace
```
