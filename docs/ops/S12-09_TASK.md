# S12-09 TASK: reviewbundle evidence auto-discovery + strict UX hardening (stopless)

- [x] 0) Branch
  - [x] `git switch -c s12-09-reviewbundle-evidence-autodetect-v1`

- [x] 1) Discovery (light, stopless)
  - [x] OBS 作成（落ちない）
  - [x] reviewbundle 実装の “実パス” を確定（for + break/continue）
  - [x] rg で evidence 関連箇所を特定し OBS に固定
  - [x] 見つからない場合は `ERROR: target_not_found` + `stop=1`（以降SKIP）

- [x] 2) Docs scaffold (light)
  - [x] docs/ops/S12-09_PLAN.md を作成（PLAN完成稿を貼付）
  - [x] docs/ops/S12-09_TASK.md を作成（本ファイル）
  - [x] docs/ops/STATUS.md を更新：S12-09 を 1% (WIP) で追加、Evidence=docs/ops/S12-09_PLAN.md

- [x] 3) Implement (light)
  - [x] reviewbundle に --evidence-report <path> を追加
  - [x] auto-detect: .local/prverify/ および docs/evidence/prverify/ から prverify_*.md を探索
  - [x] (優先) ファイル名に HEAD sha を含む候補 → 辞書順最大を採用
  - [x] (fallback) 全候補から辞書順最大を採用
  - [x] 候補の内容に HEAD sha（12桁prefix）が含まれることを保証 (content match)
  - [x] INFO で「選定理由」と「採用パス」を1行固定
  - [x] strict: evidence が見つからない場合
    - [x] ERROR: evidence_required mode=strict を出す
    - [x] stop=1 を立てる
    - [x] strict tarball を生成しない（不生成が真実）
  - [x] wip: evidence optional 維持（無い場合は INFO のみ）

- [x] 4) Tests (split; CPU守る)
  - [x] ターゲット限定で go test（go test ./... 禁止）
  - [x] 例: GOMAXPROCS=2 go test ./cmd/reviewbundle -p 1 -count=1

- [x] 5) Minimal verify (split)
  - [x] ケースA: evidence present
    - [x] go run ./cmd/reviewbundle create --mode strict --evidence-report <path> → OK & tarball生成
  - [x] ケースB: evidence missing
    - [x] go run ./cmd/reviewbundle create --mode strict → ERROR & stop=1 & tarball不生成
  - [x] stdout contract: OK: phase=end stop=<0|1> を必ず含む

- [ ] 6) Commit / Push / PR (light)
  - [ ] commit
  - [ ] push
  - [ ] gh pr create
  - [ ] PR番号確定後：docs/pr/PR-XX-s12-09-reviewbundle-evidence-autodetect.md 作成
  - [ ] 必要なら STATUS Evidence を更新
