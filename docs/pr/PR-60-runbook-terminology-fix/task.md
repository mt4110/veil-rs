# PR60 Task

- [ ] `rg -n "subshell" docs/runbook/always-run.md` を確認（定義セクション以外に出たら修正してSTOP）
- [ ] `nix run .#prverify` を実行して PASS
- [ ] prverify report を `docs/evidence/prverify/` に永続化
- [ ] `nix run .#check` を実行して PASS（COCKPIT）
- [ ] SOT の `<FILL_PRVERIFY_REPORT>` を実ファイルパスに置換
- [ ] docs-only を確認して commit & push
