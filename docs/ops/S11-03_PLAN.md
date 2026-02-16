# S11-03 PLAN — Review Bundle Go Hardening (deterministic + verify + no-leak)

## Mission (S11-03の真の目的)
Review Bundle を「監査に耐える契約成果物 (contract artifact)」に昇格させる。
= create が決定論で生成し、verify が機械的に正当性を証明でき、かつホスト情報を一切漏らさない。

## Scope
- ops/ci/review_bundle.sh を Go 実装へ置換または shim 化（最終的に contract の canonical 実装は Go）
- review bundle contract を docs で固定（仕様が先、実装が後）
- verify コマンドで “この bundle は contract を満たす” を証明

## Non-Goals
- prverify 自体の意味/判定基準は変更しない（ただし evidence を bundle に「正しく束縛」する）
- ネットワークアクセスを追加しない
- OSの tar/libarchive に依存しない（xattr漏洩の根絶）

## Canonical Contract v1 (normative)
### Determinism
- Entry ordering: bundle path の辞書順（bytewise lexicographic）で完全固定
- Timestamp:
  - SOURCE_DATE_EPOCH があればそれ（秒）
  - 無ければ git HEAD commit time (%ct)（秒）
  - tar header mtime = epoch（小数禁止）
  - gzip header modtime = epoch（OS/Extra/Name/Comment も固定）
- Header identity:
  - uid=0 gid=0 uname="" gname=""
- No host leakage:
  - xattr を含む pax key を生成しない（LIBARCHIVE.*, SCHILY.xattr.* 等が出たら fail）
  - 絶対パス/.. 参照は禁止（zip-slip類）

### Evidence binding (監査の要)
- strict:
  - clean 必須
  - prverify evidence は HEAD に束縛されていなければ error
  - “最新を入れた” fallback を禁止
- wip:
  - dirty 許可
  - evidence 未束縛なら warnings.txt に明記（曖昧さを消さず、露出させる）

### Manifest / Checksums (自己参照破綻を避ける)
- review/meta/SHA256SUMS:
  - bundle内全ファイルの sha256 を列挙するが、SHA256SUMS 自身は除外
- review/meta/SHA256SUMS.sha256:
  - SHA256SUMS の sha256 を1行で持つ

### Required layout (minimum)
- review/INDEX.md
- review/meta/contract.json (contract version / mode / base/head / epoch / tool version / warnings count)
- review/meta/SHA256SUMS
- review/meta/SHA256SUMS.sha256
- review/patch/series.patch
- review/evidence/** (strict では必須)

### Modes
- strict:
  - git status clean required
  - evidence bound to HEAD required (missing => error)
  - files snapshot は原則 HEAD blob から生成（ホスト依存排除）
- wip:
  - dirty allowed
  - include index/worktree patches
  - evidence missing/binding不可 => warning + 明示

## CLI shape (cmd/reviewbundle)
- reviewbundle create --mode {strict|wip} [--out <path>|--out-dir <dir>] [--base <rev>] [--epoch <sec>]
- reviewbundle verify <bundle.tar.gz>

## Implementation strategy (決定論の骨)
- tar/gzip は Go 標準ライブラリで直書き（外部 tar 禁止）
- epoch の決定:
  if SOURCE_DATE_EPOCH set => use it
  else => git show -s --format=%ct HEAD
- file 内容の取り出し:
  strict => git object から読む（git show HEAD:<path> 等）
  wip    => HEAD snapshot + patch で差分を表現（必要に応じて index snapshot）
- allowlist 方式:
  bundle に入れるパスを contract allowlist に限定し、未知の混入は verify で fail

## Gates
- go test ./... (PASS)
- nix run .#prverify (PASS)

## Exit Criteria
- 同一入力（同一epoch, 同一HEAD）で create を2回実行して tar.gz が byte-identical
- verify が contract 違反（xattr/pax leak, uid/gid leak, unordered, wrong epoch, evidence unbound）を確実に検知

## Discovery Results (Phase 1 Truth)
- Existing Implementation: `ops/ci/review_bundle.sh` (Found)
- Existing Go Implementation: None found (clean slate)
- Review Bundle Output: `.local/review-bundles` (defined in `ops/ci/review_bundle.sh`)
- Evidence Path (.local): `.local/prverify` (used by `review_bundle.sh` for auto-detection)
- Evidence Path (docs): `docs/evidence/prverify` (found via ls-files, committed evidence)
