# PR55 Task — pre-commit md file-url guard (no .sh)

## Goal

ステージ済み Markdown（`.md`）に raw `file:` immediately followed by `//` が含まれていたら commit を **ブロック**する。
ただし **新規 `.sh` を増やさない**（CI: No Shell Scripts を維持）。

## Acceptance Criteria

* staged `.md` に raw 連続文字列が含まれると commit が止まる（行番号表示）
* `.githooks/pre-commit` が entry point のまま
* `git ls-files '*.sh'` が増えていない
* `cargo test --workspace` PASS
* `nix run .#prverify` PASS
* evidence + SOT + plan/task が整合

## Steps

1. `.sh` 根絶（rename + pre-commit参照更新 + chmod）
2. hook テスト（Negative/Positive）
3. `cargo test --workspace` → `nix run .#prverify`
4. evidence 保存 → commit
   