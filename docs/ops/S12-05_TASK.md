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
