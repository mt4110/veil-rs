# S12-04 PLAN — CI Repro Ritual Capsule (Go)

## Goal
CI再現儀式（go test → prverify → strict create → strict verify）を、Goでカプセル化して「忘れられない形」に固定する。

- shell手順を増やさない（実行の主導はGo）
- 観測ログを `.local/obs/ci_*` に出す（毎回安定した命名）
- 失敗してもプロセスを落とさない（exit/例外/終了コードによる制御は禁止）
- 重い処理は分割できる（step実行が第一級）

## Scope
- `cmd/prkit` に `ci-repro` サブコマンドを追加
- 4ステップを提供: `go-test`, `prverify`, `strict-create`, `strict-verify`
- ログ/サマリ/STATUSスナップショットのI/O仕様を固定

## Non-goals
- GitHub Actions API / PR操作の自動化
- CIの完全再現（runner OS差分まで一致）
  - 目的は「忘れない・再現しやすい・証跡が残る」

---

## CLI Interface (Contract)

### Commands
- `prkit ci-repro run` — staged 実行（default: go-test + prverify のみ）
- `prkit ci-repro step <name>` — 1ステップだけ実行

### Flags
| Flag            | Default              | Description                   |
| --------------- | -------------------- | ----------------------------- |
| `--out-dir`     | `.local/obs`         | 出力先                        |
| `--run-id`      | `<UTC_TS>_<gitsha7>` | ランID（テストでは `fixed`）  |
| `--with-strict` | `false`              | strict-create/verify も含める |

---

## Output Contract (I/O Spec)

### File naming
`<prefix> = ci_<run_id>`

| File            | Pattern                                        |
| --------------- | ---------------------------------------------- |
| Summary         | `<out-dir>/<prefix>_summary.md`                |
| Step 01         | `<out-dir>/<prefix>_step_01_go_test.log`       |
| Step 02         | `<out-dir>/<prefix>_step_02_prverify.log`      |
| Step 03         | `<out-dir>/<prefix>_step_03_strict_create.log` |
| Step 04         | `<out-dir>/<prefix>_step_04_strict_verify.log` |
| STATUS snapshot | `<out-dir>/<prefix>_status_snapshot.txt`       |

### run_id format
- default: `YYYYMMDDTHHMMSSZ_<gitsha7>` (UTC, `git rev-parse --short=7 HEAD`)
- `--run-id` 指定時はそのまま使う

---

## Summary format (deterministic)

```md
# ci-repro summary

## Run
- run_id: <run_id>
- timestamp_utc: <YYYYMMDDTHHMMSSZ>
- git_sha: <full_sha_or_short>
- git_tree: <CLEAN|DIRTY|UNKNOWN>
- out_dir: <out_dir>
- command: <argv as single line>

## Steps
| idx | step          | status | started_utc | ended_utc | duration_ms | log_file |
| --- | ------------- | ------ | ----------- | --------- | ----------- | -------- |
| 01  | go-test       | ...    | ...         | ...       | ...         | ...      |
| 02  | prverify      | ...    | ...         | ...       | ...         | ...      |
| 03  | strict-create | ...    | ...         | ...       | ...         | ...      |
| 04  | strict-verify | ...    | ...         | ...       | ...         | ...      |

## Final
- overall: <OK|ERROR>
- error_steps: <comma-separated or NONE>

## Files
- summary: <path>
- status_snapshot: <path>
```

Rules: status = OK/ERROR/SKIP, 表は固定4行、log_file は SKIP でも作る

---

## Step Contract

### SKIP conditions
- `blocked by previous ERROR`
- strict step を `--with-strict` なしで `run` → SKIP
- `git_tree=DIRTY` → prverify/strict-* は SKIP（go-test は OK）
- コマンド未検出 → ERROR → 以後 blocked SKIP

### Commands (pinned)
| Step          | Command                                                  |
| ------------- | -------------------------------------------------------- |
| go-test       | `nix develop -c go test ./...`                           |
| prverify      | `nix run .#prverify`                                     |
| strict-create | `nix develop -c go run ./cmd/reviewbundle strict create` |
| strict-verify | `nix develop -c go run ./cmd/reviewbundle strict verify` |

---

## STATUS snapshot contract
- メタ: timestamp_utc, run_id, git sha
- `docs/ops/STATUS.md` から `| S12-` 行を抜粋
- 無ければ `ERROR: no S12 rows found`（プロセスは落とさない）

---

## Safety Rules (Non-negotiable)
- `os.Exit`, `log.Fatal`, `panic` 禁止
- **`flag.ContinueOnError` 必須** — ci-repro の FlagSet は絶対に os.Exit しない
- 子プロセス失敗 → ERROR 記録 → Go継続（最終 exit code 0）
- step 実行は **`ExecRunner` 経由** — テストでは `FakeExecRunner` で差し替え
- テストは `--run-id fixed` 必須、外部コマンド（nix等）は一切呼ばない

---

## --with-strict セマンティクス（確定）
- `run`: `--with-strict` が無い → step03/04 は SKIP（安全寄り）
- `step strict-create|strict-verify`: ユーザーが明示指定 → 実行対象（ただし DIRTY 等の安全 SKIP は適用）
- 「step は意図が明確」「run は安全寄り」

---

## ログ先頭ヘッダー（固定フォーマット）
各 step log の先頭に必ず：
```
cmd: nix develop -c go test ./...
reason: -
started_utc: 20260220T001234Z
ended_utc: 20260220T001245Z
```
SKIP の場合: `reason: blocked by previous ERROR`

---

## Atomic Write
- summary / status_snapshot は atomic write（tmp + rename）で書く
- 中途半端なファイルを観測されない保証
