# S12-01 TASK — Strict Evidence Binding (Local-first)

## A) Path Confirmation (cheap)
- [x] `git ls-files | rg '^cmd/reviewbundle/(create\.go|main\.go|verify_binding_test\.go|hermetic_repo_test\.go)$'`
- [x] `git ls-files | rg '^docs/ops/(S12-01_PLAN\.md|S12-01_TASK\.md|STATUS\.md)$'`

## B) Implement (code)
### B1) strict evidence: local-first
- [x] Edit: `cmd/reviewbundle/create.go`
  - strict の evidence 探索で `.local/prverify/` を優先
  - `prverify_*.md` を対象に HEAD(64hex) を含む最初の“最新”を採用
  - 見つからない場合は `E_EVIDENCE`（`nix run .#prverify` を促す）

### B2) no dead tar
- [x] Edit: `cmd/reviewbundle/create.go`
  - tar は tmp に作る
  - tmp を self-audit / verify して PASS なら rename
  - FAIL なら tmp を削除（死体tar禁止）

## C) Tests (light)
- [x] Add/Update: `cmd/reviewbundle/hermetic_repo_test.go`
  - TestCreate_StrictLocalEvidence（hermetic repo + .local/prverify を偽造して strict 成功を保証）
- [x] Keep: `cmd/reviewbundle/verify_binding_test.go`
  - verify の厳格条件（HEAD 含有）を維持する

## D) Unit Gate
- [x] `go test ./cmd/reviewbundle`

## E) Docs
- [x] Update: `docs/ops/S12-01_PLAN.md`（デッドロック背景 + local-first + tmp self-audit を明記）
- [ ] Update: `docs/ops/S12-01_TASK.md`（operator 手順も local-first に統一）

## F) Manual Integration (heavy; split)
### F1) generate evidence
- [ ] `nix run .#prverify`
- [ ] `ls -lt .local/prverify | sed -n '1,30p'`
- [ ] `git rev-parse HEAD`

### F2) strict create
- [ ] `go run ./cmd/reviewbundle create --mode strict --out-dir .local/review-bundles`
- [ ] `ls -lt .local/review-bundles | sed -n '1,20p'`

### F3) strict verify
- [ ] `BUNDLE_STRICT="$(ls -t .local/review-bundles/*_strict_*.tar.gz | head -n 1)"; echo "BUNDLE_STRICT=$BUNDLE_STRICT"`
- [ ] `go run ./cmd/reviewbundle verify "$BUNDLE_STRICT"`

### F4) evidence inside tar (cheap)
- [ ] `tar -tzf "$BUNDLE_STRICT" | rg 'review/evidence/prverify/prverify_.*\.md'`

## G) STATUS
- [ ] Update: `docs/ops/STATUS.md`
  - S12-01 row: `99% (Review)`（PR open + CI pass + 手元の F が PASS になったら）
  - Last Updated のみ更新（テーブル順固定）
