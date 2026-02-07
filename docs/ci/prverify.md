# PR verify (local) — one-command routine

このドキュメントは **「検証して、証拠を書いて、戻し方も書く」** をワンコマンド化するための手順です。

## なぜ必要か（未来の自分がコケないため）

手順書に **今のCLIに存在しないオプション** が混ざると、未来の自分が高確率でコケます。

## Drift Check
`prverify` includes a **Drift Check** to ensure consistency between:
1. **CI Configuration** (`.github/workflows/ci.yml`): Verifies critical security steps (using `ops/ci` scripts, generating logs, uploading artifacts) are present.
2. **Documentation** (`docs/guardrails/`): Verifies that policies like `SQLX_OFFLINE=true` and `ops/ci` exceptions are documented.
3. **Source of Truth (SOT)** (`docs/pr/`): Verifies that the active SOT (e.g., v0.22.0) exists and correctly references evidence requirements.

If `prverify` fails at the Drift Check stage, it means the code/CI implementation has diverged from the documented guardrails, and must be realigned.
- 例: `scan --rules ...` / `scan -p ...` → **scan にそのフラグが無い**
- 例: `filter --pack jp` → **filter にそのフラグが無い**

veil は「コマンドラインで pack を直接指定する」のではなく、
基本は **init が生成する設定（config）でルールや挙動が決まる** という思想です。
だから、再現手順も「CLIフラグ列」ではなく **設定駆動 + テスト駆動** で固めるのが安全。

## 使い方

### 1) ワンコマンド（推奨）

```bash
nix run .#prverify
```

これがやること：
- `cargo test -p veil-cli --test cli_tests`（trycmd の P0 smoke）
- `cargo test --workspace`（全体テスト）
- 実行ログを **Markdownレポート** に保存（Notes / Evidence / Rollback つき）

### 2) 出力ファイル

レポートは下記に保存されます：

- `.local/prverify/prverify_<timestamp>_<sha>.md`

PR本文やレビューコメントに貼るなら、このファイルをコピペ。

## 仕組み（設計）

- 生成物（レポート）は `.local/` 配下に集約して **git管理外** にする（`.gitignore` 済み）
- レポートは Markdown で
  - 実行したコマンド
  - 実行ログ
  - Notes / Evidence
  - Rollback
  を固定フォーマットで残す

## トラブルシュート

- `nix run` が重い → 初回はNixキャッシュが温まってないだけのことが多いです。
- `git` が無いと言われる → `nix run .#prverify` 経由ならOK（`git` も同梱）。

## ロールバック（安全に戻す）

基本はこれでOK：

```bash
git revert <commit>
```

複数コミットをまとめて戻すなら：

```bash
git revert <oldest_commit>^..<newest_commit>
```
