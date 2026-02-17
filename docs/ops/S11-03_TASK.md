# S11-03 TASK — Review Bundle Go Hardening (ordered)

## Phase 0 — Safety snapshot
- [x] cd "$(git rev-parse --show-toplevel)"
- [x] git status -sb（dirtyなら理由を1行）
- [x] git rev-parse --abbrev-ref HEAD（s11-03-*）

## Phase 1 — Path discovery (truth; resolve real paths)
- [x] ops/ci/review_bundle.sh の有無を確認
- [x] 既存の review bundle 出力先（.local/配下など）を特定
- [x] prverify evidence の格納パスを特定（docs/evidence/prverify 等）
- [x] 結果を docs/ops/S11-03_PLAN.md に追記（“repoの真実” として固定）

## Phase 2 — Contract doc (first; implementation follows)
- [x] docs/ops/REVIEW_BUNDLE.md を新規作成
  - [x] Contract v1 (MUST/SHOULD)
  - [x] Layout
  - [x] Determinism rules
  - [x] Evidence binding rules
  - [x] Manifest rules（SHA256SUMS 自己参照除外 + sha256封印）

## Phase 3 — Go CLI scaffold
- [x] cmd/reviewbundle/ を作成
- [x] create サブコマンド骨格（引数/戻り値/エラー規約）
- [x] verify サブコマンド骨格（allowlist + checksums + contract）

## Phase 4 — Deterministic tar.gz writer (no leak)
- [x] epoch 決定（SOURCE_DATE_EPOCH else git %ct）
- [x] gzip header: modtime/OS/Extra/Name/Comment を固定
- [x] tar header: uid/gid/uname/gname を固定（0/0/空）
- [x] tar header: mtime を epoch 秒（小数禁止）
- [x] entry order: bundle path を bytewise lexicographic で固定
- [x] xattr/pax leak:
  - [x] LIBARCHIVE.* / SCHILY.xattr.* が出ないことを verify で検出（出たら error）

## Phase 5 — Evidence binding hardening
- [x] strict:
  - [x] evidence を “HEAD に束縛” できないなら error
  - [x] “最新 fallback” 禁止
- [x] wip:
  - [x] evidence 未束縛なら warnings.txt に明記（曖昧性を露出）

## Phase 6 — Manifest (checksums)
- [x] review/meta/SHA256SUMS を生成（自分自身は除外）
- [x] review/meta/SHA256SUMS.sha256 を生成（封印）

## Phase 7 — Determinism tests
- [x] 同一入力で create を2回 → tar.gz が byte-identical のテスト
- [x] uid/gid/uname/gname が漏れないテスト
- [x] mtime/epoch が固定のテスト
- [x] entry order が固定のテスト
- [x] evidence binding の strict fail / wip warn のテスト

### 5.2 Minimal synthetic fixtures (in-memory tar.gz)
- [x] verify_test.go で "最小の正しいbundle" を生成して verify PASS を確認
- [x] determinism_test.go で "同一入力 => 同一検査結果" を保証（検査が環境依存しない）

## Phase 7.5 — Fix known-bad test input (synthetic)
- [x] TestVerify_FailsOnKnownBadBundle を ForgeBundle で合成
- [x] .local 依存の排除
- [x] エラーコードの許容判定（E_PAX, E_XATTR, E_IDENTITY, E_TIME）

## Phase 8 — Wire-up
- [x] ops/ci/review_bundle.sh を shim 化（Go 呼び出しのみ）or 廃止
- [x] flake.nix に reviewbundle app/package を追加（nix run .#reviewbundle）

## Phase 9 — Gates
- [x] go test ./... (PASS)
- [x] nix run .#prverify (PASS)

## Phase 10 — Commit & PR
- [x] git add docs/ops cmd/reviewbundle ops/ci flake.nix
- [x] commit
- [x] push
- [x] PR 作成（S11-03）

### 4.2 Implement streaming verifier (continued)
- [x] verify.go:
  - [x] gzip header capture (mtime/name/comment/extra/os)
  - [x] tar stream scan:
    - [x] ordering (bytewise lexicographic)
    - [x] path safety (no abs / no .. / no NUL / clean)
    - [x] type allowlist (dir/file/symlink only)
    - [x] uid/gid/uname/gname fixed
    - [x] pax allowlist only; forbid xattr/provenance; forbid pax time keys (C2)
    - [x] mtime: nanos==0 AND all entries same unix seconds
    - [x] collect required layout presence (C2)
    - [x] collect sha256 for all non-dir entries (C2)
    - [x] parse contract.json (epoch_sec/mode/head_sha/warnings_count) (C3)
    - [x] parse SHA256SUMS + seal + warnings.txt (C3)
    - [x] post-conditions: manifest verify, epoch check, layout check, evidence binding (C3)
