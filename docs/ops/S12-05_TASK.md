# S12-05 TASK (v1)

- [x] Create branch: `s12-05-cirepro-runner-alignment-v1`
- [ ] Path discovery logs saved under `.local/obs/s12-05_*/`
- [x] Baseline: `go test ./...` (log)
- [x] Baseline: `nix run .#prverify` (log)
- [x] Baseline: ci-repro representative run(s) (log)

- [x] Refactor: ci-repro -> prkit runner entry alignment
- [x] Refactor: deps/DI alignment (thread deps to core)
- [x] Tests: add/adjust minimal tests for DI seams (if applicable)
- [x] Docs: update run instructions (ci-repro via prkit runner)
- [x] SOT: update `docs/ops/STATUS.md` for S12-05 + Last Updated + Evidence

- [x] Final verify: `go test ./...`
- [x] Final verify: `nix run .#prverify`
- [x] PR body: SOT/証拠スタイル（ガチガチ版＋短縮版）

---

## Phase 2: Copilot Review Fixups

- [x] OBS作成（UTC）
- [x] PR番号の自動確定（env PR が無ければ gh から拾う）
- [x] Copilotレビュー採取（JSON保存）
- [x] Copilotレビュー抽出サマリ生成（Markdown保存）
- [x] 指摘の分類（docs / correctness / determinism / “要判断”）
- [x] 修正（1グループずつ）
- [x] gofmt（触ったファイルだけ）
- [x] 軽いテスト（パッケージ限定）
- [x] 重いテスト（最後に1回だけ）
- [x] nix run .#prverify
- [ ] clean tree 確認 → strict reviewbundle create（最後に1回）
- [ ] docs/pr のSOT文書を実値で更新（PR番号、SHA、証拠パス、bundle sha256）
- [ ] docs/ops/STATUS.md 更新（S12-05 99% Review + Evidence）
- [ ] OK: phase=end
