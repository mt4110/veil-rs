# S12-10 PLAN — Reviewbundle Pack Contract v1

## 状態
- S12-10: 進行中（設計フェーズ）
- ゴール: 「bundle が存在する = verify できる」を契約（manifest + verify）として固定する

## 設計テーマ（Working Theory）
「reviewbundle を“配布物”として完成させる」  
= 生成できるだけでなく、**検証できる・将来も読み取れる・掃除できる** を契約化する。

## 制約（絶対）
- exit系全面禁止（終了コードで制御しない）
- 失敗は `ERROR:` を print、`stop=1` を立てる（stopless）
- 重い処理を強制しない（prverify/cargo/go test ./... を verify の内部で回さない）
- verify は bundle 単体で自己完結（ネットワーク不要 / repo全体スキャン不要）

## 用語
- bundle: reviewbundle の生成物（ディレクトリ or アーカイブ展開結果）
- manifest v1: bundle の“目録” + 最低限の互換情報
- schema: manifest の形式（将来の互換維持のために明文化）
- verify: bundle と manifest の整合を軽く検証するコマンド（stopless）

---

## ゴール（案）
- strict bundle は **manifest v1 を必須**にする（bundle には必ず入る）
- `reviewbundle verify`（または同等）で以下を **軽く**検証できる:
  - manifest が存在し、最低限の schema を満たす
  - manifest の file list と実ファイル集合が一致（missing/extra を検出）
  - size / sha256（または budget 内での sha256）一致
  - evidence_report の参照が契約どおり（禁止スキーム/絶対パス/.. を含まない等）
- 改ざん（1ファイル変更 / manifest 欠落 / sha不一致）で `ERROR` + `stop=1`（stopless）

## 非ゴール（案）
- 署名 / 鍵ローテ / 供給網セキュリティ強化は別フェーズ（S12-10ではやらない）
- prverify を verify が勝手に実行しない
- リポジトリ全体をスキャンしない（bundle だけを見る）

---

## Pack Contract v1（規範）
### MUST（必須）
- bundle には **manifest v1** が存在する
- manifest は **bundle 内のファイル集合を閉じる**（基本: extra/missing をエラーにする）
- manifest 内の path は相対パスのみ
  - 絶対パス禁止
  - `..` 禁止（parent traversal 禁止）
  - OS依存のドライブ表現は禁止（例: `C:\`）
- verify は **stopless**（失敗しても exit code で制御しない）

### SHOULD（推奨）
- manifest の files は path ソート（再現性・差分最小化）
- sha256 は lower-hex
- bundle の生成は「上書きしない」
  - 既存出力がある場合は別IDを採る or cemetery に退避してから新規生成

### MAY（任意）
- verify に budget（最大ファイル数/最大バイト数）を持たせ、超過時は `ERROR: budget_exceeded` として止める
  - 端末フリーズ防止のための安全弁（契約違反を“隠さない”）

---

## 実パス固定・定数定義（Discovery 反映）
- `BUNDLE_ROOT` = `review/`
- `CONTRACT_PATH` = `review/meta/contract.json`
- `HASH_MANIFEST_PATH` = `review/meta/SHA256SUMS`
- `HASH_MANIFEST_DIGEST_PATH` = `review/meta/SHA256SUMS.sha256`
- `EVIDENCE_PREFIX` = `review/evidence/`
- `EVIDENCE_PRVERIFY_DIR` = `review/evidence/prverify/`
- `INDEX_PATH` = `review/INDEX.md`
- `PATCH_PATH` = `review/patch/series.patch`

## manifest の定義
JSON の形式定義は置換され、以下を**事実上の manifest** とする：
- 本体: `SHA256SUMS`（`sha256_hex  相対path` のリスト）
- シール: `SHA256SUMS.sha256`（`SHA256SUMS` 自身の改ざん検知）

## evidence の定義（contract.json と整合）
`contract.json` の内容を SSOT と定義し、これに違反しないようにする。
- `strict`: `evidence.required=true` を必須とする（`present=true`）
- `wip`: `evidence.required=false` を許容する

## verify（stopless）設計：疑似コード（分岐/停止条件）

ここは “Plan.md は疑似コード” の本体。exit禁止・嘘禁止。

```text
state:
  stop = 0
  errors = []
  warnings = []

inputs:
  bundle_path

constants (fixed by contract):
  BUNDLE_ROOT = "review/"
  CONTRACT_PATH = "review/meta/contract.json"
  HASH_MANIFEST_PATH = "review/meta/SHA256SUMS"
  HASH_MANIFEST_DIGEST_PATH = "review/meta/SHA256SUMS.sha256"
  EVIDENCE_PREFIX = "review/evidence/"
  EVIDENCE_PRVERIFY_DIR = "review/evidence/prverify/"

  MAX_FILES = 5000
  HASH_BUDGET_BYTES = 268435456  # 256MiB
  EVIDENCE_SCAN_BYTES = 65536    # 64KiB (partial scan)

helpers:
  error(msg): append(errors, msg)
  warn(msg): append(warnings, msg)

  # NOTE: docs-guard safe:
  # do NOT write raw "://". Detect by checking "file:" and then " // " adjacency.
  contains_file_scheme_like(s):
    # conceptual: returns true if s contains the substring "file:" followed immediately by "//"
    # implementation may scan for "file:" then check next 2 chars are both '/'
    return has_adjacent(s, "file:", "//")

  contains_parent_traversal(path):
    # reject ".." segments
    return path contains "/../" or path starts with "../" or path ends with "/.."

  path_invalid(path):
    if path is empty: return true
    if path starts with "/": return true                  # absolute
    if contains_parent_traversal(path): return true       # parent traversal
    if path matches "^[A-Za-z]:\\": return true           # windows drive
    return false

try:
  # 1) input sanity
  if bundle_path is empty:
    error("missing_bundle_path"); stop=1
  else if not exists(bundle_path):
    error("bundle_not_found"); stop=1
  else if not is_dir(bundle_path):
    error("bundle_not_dir"); stop=1
  else:
    OK

  # 2) required files exist
  if stop == 0:
    contract_path = resolve(bundle_path, CONTRACT_PATH)
    manifest_path = resolve(bundle_path, HASH_MANIFEST_PATH)
    seal_path = resolve(bundle_path, HASH_MANIFEST_DIGEST_PATH)

    if not exists(contract_path):
      error("contract_missing"); stop=1
    if not exists(manifest_path) or not exists(seal_path):
      error("manifest_missing"); stop=1

  # 3) load + validate contract.json (SSOT)
  if stop == 0:
    contract = load_json(contract_path)

    # minimal contract checks (do NOT require schema beyond what you use)
    if contract.contract_version is empty:
      error("contract_version_missing"); stop=1
    if contract.mode not in ["strict", "wip"]:
      error("contract_mode_invalid"); stop=1

    # evidence rules from SSOT
    # strict: evidence.required must be true
    if stop == 0 and contract.mode == "strict":
      if contract.evidence.required != true:
        error("strict_requires_evidence_required_true"); stop=1

    # sanity for prefixes (do not overfit)
    if stop == 0:
      if contract.evidence.path_prefix is empty:
        warn("evidence_path_prefix_missing_in_contract")

  # 4) verify SHA256SUMS seal (SHA256SUMS.sha256)
  if stop == 0:
    expected = load_seal_hex(seal_path)           # should be hex sha256 of SHA256SUMS
    actual = sha256_hex_of_file(manifest_path)
    if expected is empty:
      error("manifest_seal_empty"); stop=1
    else if actual != expected:
      error("manifest_seal_broken"); stop=1

  # 5) parse SHA256SUMS entries
  # format: "<sha256_hex><spaces><path>"
  if stop == 0:
    entries = parse_sha256sums(manifest_path)
    # entries: list of {path, sha256_hex}

    if entries is empty:
      error("manifest_empty"); stop=1
    else if count(entries) > MAX_FILES:
      error("budget_exceeded:max_files"); stop=1

  # 6) compute closed world file set (S_allow) and compare with actual
  if stop == 0:
    S_hash = set()
    for e in entries:
      if path_invalid(e.path):
        error("path_invalid:" + e.path); stop=1; break
      # Require paths are under review/ (contracted bundle root)
      if not e.path starts with BUNDLE_ROOT:
        error("path_not_under_bundle_root:" + e.path); stop=1; break
      add S_hash e.path

    if stop == 0:
      S_meta = set([HASH_MANIFEST_PATH, HASH_MANIFEST_DIGEST_PATH])
      S_allow = union(S_hash, S_meta)

      # actual_files should be relative paths from repo root inside the bundle
      S_actual = list_files_relative(bundle_path)   # includes "review/..." entries
      # IMPORTANT: ensure it includes meta files too; do not exclude by default

      missing = set_diff(S_allow, S_actual)
      extra = set_diff(S_actual, S_allow)

      if missing not empty:
        error("file_missing:" + join_sorted(missing, ",")); stop=1
      if extra not empty:
        error("file_extra:" + join_sorted(extra, ",")); stop=1

  # 7) verify sha256 of each entry with budget (size-based)
  if stop == 0:
    bytes_hashed = 0

    # stable order
    entries_sorted = sort_by_path(entries)

    for e in entries_sorted:
      # size budget uses actual file size
      size = stat_size(resolve(bundle_path, e.path))
      if size < 0:
        error("stat_failed:" + e.path); stop=1; break

      if bytes_hashed + size > HASH_BUDGET_BYTES:
        error("budget_exceeded:hash_bytes"); stop=1; break

      actual_sha = sha256_hex_of_file(resolve(bundle_path, e.path))
      if actual_sha != e.sha256_hex:
        error("sha256_mismatch:" + e.path); stop=1; break

      bytes_hashed = bytes_hashed + size
      continue

  # 8) evidence sanity (targeted; do NOT ban https)
  if stop == 0:
    # Strict evidence: require at least one file under review/evidence/prverify/
    if contract.mode == "strict":
      ev_files = list_files_under(bundle_path, EVIDENCE_PRVERIFY_DIR)
      if count(ev_files) < 1:
        error("evidence_missing:prverify"); stop=1

    if stop == 0:
      # Partial scan: read first EVIDENCE_SCAN_BYTES of each evidence file
      ev_scan_targets = list_files_under(bundle_path, EVIDENCE_PREFIX)
      for p in ev_scan_targets:
        chunk = read_first_bytes_as_text(resolve(bundle_path, p), EVIDENCE_SCAN_BYTES)

        # forbid file scheme like (guard-safe detection)
        if contains_file_scheme_like(chunk):
          error("forbidden_uri_scheme:file:" + p); stop=1; break

        # forbid absolute path smell (keep simple; avoid false positives)
        if contains(chunk, "/Users/") or contains(chunk, "C:\\"):
          error("forbidden_absolute_path:" + p); stop=1; break

        # forbid parent traversal mention
        if contains(chunk, "/../") or contains(chunk, "../"):
          error("forbidden_parent_traversal:" + p); stop=1; break

        continue

catch any:
  error("unexpected_exception"); stop=1

finally:
  # summary: do NOT rely on exit codes
  if stop == 0:
    print("OK: verify=pass")
  else:
    for msg in errors:
      print("ERROR: " + msg)
  for msg in warnings:
    print("WARN: " + msg)
  print("OK: phase=end stop=" + stop)
```

## DoD（Definition of Done）
- strict bundle: manifest v1 が入り、verify が通る
- 改ざんケースで verify が ERROR + stop=1 を出す（終了コードに依存しない）
- docs（PLAN/TASK/STATUS）が一致し、嘘がない
