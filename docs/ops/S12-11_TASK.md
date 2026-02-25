# S12-11 TASK: reviewbundle verify SSOT v1

> ルール：exit系禁止。set -e 禁止。return 非ゼロ禁止。  
> 成否は出力テキスト（OK/ERROR/SKIP と stop=）で判定。  
> 重い処理は分割。OBS を必ず残す。

---

## 0) 前提（進捗）
- S12-10：100%（Merged PR #95）
- S12-11：0% → このPR開始で 1%（WIP）

---

## 1) Kickoff Discovery（軽量・OBS）
- [ ] discovery を実行して OBS を作る（既に実行済みなら SKIP して OBS パスだけ残す）

```bash
bash -lc '
ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
STOP="0"

if [ -z "$ROOT" ]; then
  echo "ERROR: not_in_repo"
  STOP="1"
else
  cd "$ROOT" 2>/dev/null || true

  TS="$(date -u +%Y%m%dT%H%M%SZ)"
  OBS=".local/obs/s12-11_discovery_${TS}"
  mkdir -p "$OBS" 2>/dev/null || true
  echo "OK: obs_dir=$OBS"

  ls -la cmd/reviewbundle 2>/dev/null | tee "$OBS/ls_cmd_reviewbundle.txt" || true

  rg -n --no-heading -S "Verify|verify|SHA256SUMS|SHA256SUMS\\.sha256|contract\\.json|budget|EVIDENCE|file:" \
    cmd/reviewbundle 2>/dev/null | tee "$OBS/rg_reviewbundle_verify_ssot.txt" || true

  rg -n --no-heading -S "review/meta/contract\\.json|review/meta/SHA256SUMS|review/meta/SHA256SUMS\\.sha256|review/evidence/prverify" \
    cmd/reviewbundle 2>/dev/null | tee "$OBS/rg_fixed_paths.txt" || true
fi

echo "OK: phase=end stop=$STOP"
'
```

- [ ] SSOT 契約 docs を観測（S12-10 の contract v1 を読む）

```bash
bash -lc '
ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
STOP="0"

if [ -z "$ROOT" ]; then
  echo "ERROR: not_in_repo"
  STOP="1"
else
  cd "$ROOT" 2>/dev/null || true

  TS="$(date -u +%Y%m%dT%H%M%SZ)"
  OBS=".local/obs/s12-11_contract_read_${TS}"
  mkdir -p "$OBS" 2>/dev/null || true
  echo "OK: obs_dir=$OBS"

  ls -la docs/ops 2>/dev/null | tee "$OBS/ls_docs_ops.txt" || true

  # 契約（v1）を “軽く” 抜粋（行数が多くても sed は軽い）
  sed -n "1,220p" docs/ops/REVIEWBUNDLE_PACK_CONTRACT_v1.md 2>/dev/null | tee "$OBS/contract_head_220.txt" || true
fi

echo "OK: phase=end stop=$STOP"
'
```

- [ ] contract.json の “実物サンプル” を repo 内から探す（あれば観測、なければ SKIP）

```bash
bash -lc '
ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
STOP="0"

if [ -z "$ROOT" ]; then
  echo "ERROR: not_in_repo"
  STOP="1"
else
  cd "$ROOT" 2>/dev/null || true

  TS="$(date -u +%Y%m%dT%H%M%SZ)"
  OBS=".local/obs/s12-11_contract_sample_${TS}"
  mkdir -p "$OBS" 2>/dev/null || true
  echo "OK: obs_dir=$OBS"

  # まずは repo 内に置かれた fixture を探す
  find . -type f -name "contract.json" 2>/dev/null | tee "$OBS/find_contract_json.txt" || true

  # 見つかったら先頭だけ読む（重くしない）
  CAND="$(find . -type f -name "contract.json" 2>/dev/null | head -n 1)"
  if [ -n "$CAND" ]; then
    echo "OK: sample=$CAND" | tee "$OBS/sample_path.txt" || true
    sed -n "1,240p" "$CAND" 2>/dev/null | tee "$OBS/contract_sample_head_240.txt" || true
  else
    echo "SKIP: contract_json_sample_not_found" | tee "$OBS/sample_path.txt" || true
  fi
fi

echo "OK: phase=end stop=$STOP"
'
```

## 2) 実装方針の確定（SSOT/閉世界/予算/出力）
- [ ] verify の最終1行フォーマットを固定（終了コード依存なし）
  - 成功：`OK: verify stop=0`
  - 失敗：`ERROR: <reason> stop=1`
- [ ] 失敗 reason を固定（最低限）
  - `budget_exceeded`
  - `missing_file`
  - `extra_file`
  - `seal_broken`
  - `sha_mismatch`
  - `evidence_forbidden`
- [ ] 固定パス（bundle root 相対）を定数化
  - `review/meta/contract.json`
  - `review/meta/SHA256SUMS`
  - `review/meta/SHA256SUMS.sha256`
  - `review/evidence/prverify`

## 3) errors.go（ErrorCode と標準出力整合）
- [ ] cmd/reviewbundle/errors.go を作る or 既存を整理
  - `type ErrorCode string`
  - `const (E_SHA256 ... )`
  - `type RBError struct { Code, Reason, Path, Detail }`
  - `func (e RBError) Line() string`（ERROR:行を作る）

OBS（軽量）：
```bash
bash -lc '
ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
STOP="0"

if [ -z "$ROOT" ]; then
  echo "ERROR: not_in_repo"
  STOP="1"
else
  cd "$ROOT" 2>/dev/null || true

  TS="$(date -u +%Y%m%dT%H%M%SZ)"
  OBS=".local/obs/s12-11_errors_discovery_${TS}"
  mkdir -p "$OBS" 2>/dev/null || true
  echo "OK: obs_dir=$OBS"

  rg -n --no-heading -S "ErrorCode|E_SHA|budget_exceeded|seal_broken|sha_mismatch" cmd/reviewbundle \
    2>/dev/null | tee "$OBS/rg_errors.txt" || true
fi

echo "OK: phase=end stop=$STOP"
'
```

## 4) SSOT 読み取り（contract + sums + seal）
- [ ] SHA256SUMS.sha256 の parse（期待hash抽出）
- [ ] SHA256SUMS の sha256 を計算し seal と一致確認
- [ ] SHA256SUMS を parse（path->hash）
  - path の正規化（.. 禁止、絶対パス禁止、bundle 外禁止）
- [ ] contract.json を読み取り（schema は discovery 結果に合わせる）
  - 最小でも “存在確認 + hash対象であること” は守る
  - 可能なら contract にあるファイル集合と sums の一致確認
  - ※ ここは “重くなる要因” なので、budget を最初から噛ませる。

## 5) 閉世界（closed world）
- [ ] filepath.WalkDir で bundle を walk（budget: files）
  - symlink は即 stop=1（閉世界を破れる）
- [ ] 実ファイル集合 vs SSOT集合の差分を出す
  - missing => `ERROR: missing_file ... stop=1`
  - extra => `ERROR: extra_file ... stop=1`
- [ ] 差分が出たら “以降の hash 検証は進めない”（止まらない＝exitしないが、工程は止める）

## 6) sha256 検証（改ざん検出）
- [ ] sums に載っている各ファイルを sha256 検証（budget: bytes）
  - mismatch => `ERROR: sha_mismatch ... stop=1`
  - 途中で budget 超過 => `ERROR: budget_exceeded ... stop=1`

## 7) evidence スキャン（狙い撃ち）
- [ ] 対象：`review/evidence/prverify/**`（存在するもののみ）
- [ ] 禁止検出（https を殺さない）
  - file スキーム：`file: + //` `file:/` `file:\`
  - `../`（ただし URL 内は基本スルーできる形で）
  - 絶対パス臭：`/Users/` `/home/` `/etc/` `/var/` `/private/` `/Volumes/` `/mnt/` 等
  - Windows：`C:\` `C:/` 等
- [ ] バイナリっぽいファイルは SKIP 可能（OBS に残す）

## 8) verify.go に統合（最終1行 + stop=）
- [ ] verify の中核を VerifyBundle(opts) 的な関数にまとめる（テスト容易化）
- [ ] recover() で panic を捕まえて `ERROR: panic_recovered stop=1` に落とす
- [ ] 最後に必ず：
  - stop==0: `OK: verify stop=0`
  - stop==1: `ERROR: <reason> stop=1`

## 9) create.go に統合（strict は自家検証）
- [ ] strict create の最後に軽量 verify を呼ぶ（同一ロジック再利用）
  - NGなら `ERROR: create_generated_invalid_bundle stop=1`
  - OKなら `OK: create stop=0`
- [ ] wip は “SSOT未完” を許すが、strict は必ず SSOT を満たす

## 10) テスト（軽量 fixture・再現性固定）
- [ ] テストケース（最低限）
  - [ ] strict OK
  - [ ] missing
  - [ ] extra
  - [ ] seal broken
  - [ ] sha mismatch
  - [ ] budget exceeded
  - [ ] evidence forbidden
  - [ ] evidence allow（https は許可）
- [ ] fixture は数KB〜数十KB
- [ ] go test は “対象パッケージ限定” で実行（重くしない）

軽量テスト実行（OBS、止まらない）：
```bash
bash -lc '
ROOT="$(git rev-parse --show-toplevel 2>/dev/null || true)"
STOP="0"

if [ -z "$ROOT" ]; then
  echo "ERROR: not_in_repo"
  STOP="1"
else
  cd "$ROOT" 2>/dev/null || true

  TS="$(date -u +%Y%m%dT%H%M%SZ)"
  OBS=".local/obs/s12-11_go_test_${TS}"
  mkdir -p "$OBS" 2>/dev/null || true
  echo "OK: obs_dir=$OBS"

  # CPU 暴れ対策：対象を絞る（./... は避ける）
  GOMAXPROCS=1 go test ./cmd/reviewbundle -count=1 -run "Test" 2>&1 | tee "$OBS/go_test_cmd_reviewbundle.txt" || true

  # 出力で真実を見る（終了コード依存なし）
  if rg -n "FAIL|panic" "$OBS/go_test_cmd_reviewbundle.txt" >/dev/null 2>&1; then
    echo "ERROR: go_test_failed stop=1" | tee "$OBS/go_test_judge.txt" || true
    STOP="1"
  else
    echo "OK: go_test_pass stop=0" | tee "$OBS/go_test_judge.txt" || true
  fi
fi

echo "OK: phase=end stop=$STOP"
'
```

## 11) docs & STATUS 更新
- [ ] `docs/ops/S12-11_PLAN.md` を追加（本ファイル）
- [ ] `docs/ops/S12-11_TASK.md` を追加（本ファイル）
- [ ] `docs/ops/STATUS.md` を更新（S12-11=1% WIP）

## 12) 仕上げ（PR）
- [ ] git status -sb を OBS に残す
- [ ] 差分を OBS に残す（git diff --stat）
- [ ] push → PR
- [ ] PR 本文に：
  - SSOT 検証の仕様（stop= と reason）
  - budget の既定値
  - テストケース一覧
