# S12-01 PLAN — Strict Evidence Binding (Local-first, No-Dirty)

## Goal
`go run ./cmd/reviewbundle create --mode strict` が、
- git repo を汚さず（cleanのまま）
- bundle 内 `review/evidence/` に **HEAD SHA を含む証拠（prverifyレポート）** を同梱でき、
- self-audit / verify を PASS できるようにする。

## Background / Problem
現状の strict は次の契約を同時に満たそうとして詰む：
1. (A) strict は git clean を要求（untracked も禁止）
2. (B) strict は bundle 内 evidence が HEAD(64hex) を含むことを要求
3. (C) evidence を repo 配下（例: `docs/evidence/...`）へコピーすると untracked が生え、(A) で失敗

結果：`E_EVIDENCE` / `E_CONTRACT` のデッドロックが発生する。
さらに、self-audit が失敗しても tar が残り、次の検証を汚す（“死体tar”）。

## Non-Negotiable Contracts
- strict は `git status --porcelain` が空（tracked/untracked なし）であること。
- strict は bundle 内 evidence が HEAD(64hex) を含むこと。
- strict の成功要件は repo を汚さずに上記を満たすこと。
- 失敗時に “死体tar” を残さない（tmp で止める）。

## Design

### Evidence Sourcing (strict)
strict の evidence 探索は次の順で行う：
1. `.local/prverify/`（ローカル実行 `nix run .#prverify` の出力）
2. 既存の repo evidence パス（互換のため）

探索条件：
- 対象ファイル：`prverify_*.md`
- 判定：ファイル内容に HEAD(64hex) が文字列として含まれること
- 複数候補がある場合：最新を選ぶ（できればファイル名の timestamp 順で決定）
- 見つからない場合：
  - `E_EVIDENCE` を返し、オペレータに `nix run .#prverify` を促すメッセージを出す。

### Bundle Creation (no dead tar)
- 出力は `outDir` 配下に tmp ファイルとして作る（同一FS上）
- tmp に対して self-audit / verify を実施
- PASS のときだけ `os.Rename(tmp, final)`（atomic）
- FAIL の場合は tmp を削除して終了（死体tarを残さない）

## Acceptance Criteria
### Unit:
- `go test ./cmd/reviewbundle` が PASS
- `TestCreate_StrictLocalEvidence` が PASS（strict が `.local/prverify` を拾う）
- 既存 verify/binding の厳格さは維持される（テストで保証）

### Manual Integration (operator):
- repo clean を確認
- `nix run .#prverify` を実行（`.local/prverify/` に HEAD 含む report 生成）
- `go run ./cmd/reviewbundle create --mode strict ...`
- `go run ./cmd/reviewbundle verify <bundle>`
- tar 内に `review/evidence/prverify/prverify_*.md` が入っており、内容が HEAD を含む

## Rollback
この PR の merge commit（または squash commit）を revert する。
