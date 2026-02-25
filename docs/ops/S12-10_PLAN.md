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

## manifest v1（暫定スキーマ案）
> 実パス・ファイル名は Discovery 後に確定する（ここは“型”）

- kind: `"reviewbundle.manifest"`
- version: `1`
- created_utc: RFC3339（例: `2026-02-25T00:00:00Z`）
- tool:
  - name / version / commit（可能なら）
- bundle:
  - format: `strict|loose`（まず strict で必須化）
  - id: 生成ID（timestamp等）
- paths:
  - evidence_report: evidence_report の相対パス
- files: 配列
  - path: 相対パス
  - size: bytes（整数）
  - sha256: lower-hex（可能なら必須）
  - role: `"evidence_report" | "doc" | "artifact" | ...`（任意）

### 例（イメージ）
```json
{
  "kind": "reviewbundle.manifest",
  "version": 1,
  "created_utc": "2026-02-25T00:00:00Z",
  "tool": { "name": "reviewbundle", "version": "0.0.0", "commit": null },
  "bundle": { "format": "strict", "id": "rb_20260225T000000Z" },
  "paths": { "evidence_report": "evidence/evidence_report.json" },
  "files": [
    { "path": "README.md", "size": 1234, "sha256": "…", "role": "doc" },
    { "path": "evidence/evidence_report.json", "size": 4567, "sha256": "…", "role": "evidence_report" }
  ]
}
```

## verify（stopless）設計：疑似コード（分岐/停止条件）

ここは “Plan.md は疑似コード” の本体。exit禁止・嘘禁止。

```text
state:
  stop = 0
  errors = []
  warnings = []

input:
  bundle_path

try:
  if bundle_path is empty:
    error("missing_bundle_path"); stop=1
  else if not exists(bundle_path):
    error("bundle_not_found"); stop=1
  else if not is_dir(bundle_path):
    error("bundle_not_dir"); stop=1
  else:
    OK

  if stop == 0:
    manifest_path = resolve(bundle_path, MANIFEST_PATH)  # ※Discovery後に固定
    if not exists(manifest_path):
      error("manifest_missing"); stop=1
    else:
      manifest = load_json(manifest_path)
      if not schema_minimal_ok(manifest):
        error("manifest_schema_invalid"); stop=1

  if stop == 0:
    # file set check（閉世界）
    actual_files = list_files(bundle_path, exclude=[MANIFEST_PATH])
    manifest_files = manifest.files[].path

    if diff(actual_files, manifest_files) has missing:
      error("file_missing"); stop=1
    if diff(actual_files, manifest_files) has extra:
      error("file_extra"); stop=1

  if stop == 0:
    # budget safety valve（端末保護）
    bytes_hashed = 0
    for each file in manifest.files sorted by path:
      if path_invalid(file.path):  # absolute / .. / drive
        error("path_invalid:" + file.path); stop=1; break
      if stop == 1:
        break

      actual_size = stat_size(bundle_path/file.path)
      if actual_size != file.size:
        error("size_mismatch:" + file.path); stop=1; break

      # sha256 は「軽さ」を守るため budget を持てる
      if bytes_hashed + actual_size > HASH_BUDGET_BYTES:
        error("budget_exceeded"); stop=1; break
      else:
        actual_sha = sha256(bundle_path/file.path)
        if actual_sha != file.sha256:
          error("sha256_mismatch:" + file.path); stop=1; break
        bytes_hashed += actual_size
        continue

  if stop == 0:
    # evidence_report sanity（禁止物）
    er = load_text(bundle_path / manifest.paths.evidence_report)
    if contains(er, "://") or contains(er, "http://") or contains(er, "https://"):
      error("forbidden_uri_scheme:file"); stop=1
    else if contains_absolute_path_like(er):
      error("forbidden_absolute_path"); stop=1
    else:
      OK

catch any:
  # 例外で落ちない。嘘をつかずERROR化。
  error("unexpected_exception"); stop=1

finally:
  print summary lines
  print "OK: phase=end stop=" + stop
```

## DoD（Definition of Done）
- strict bundle: manifest v1 が入り、verify が通る
- 改ざんケースで verify が ERROR + stop=1 を出す（終了コードに依存しない）
- docs（PLAN/TASK/STATUS）が一致し、嘘がない
