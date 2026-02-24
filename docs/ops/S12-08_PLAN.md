# S12-08 PLAN: S12-07 SOT closeout (STATUS update + PR92 doc fix)

## Goal
S12-07 の成果を **SOT（docs/ops/STATUS.md + docs/pr）** に刻み、
「長期的に破綻しない」状態で閉じる。

- PR #92 は **Merged**（merge commit: 33ab2bd / checks 11 passed / branch deleted）まで確定済み
- 本フェーズ S12-08 は **最小コスト（docs中心）** で 1PR で閉じる

## Non-Goals
- 重い検証（nix / full test / prverify 実行）はやらない（CIが証拠）
- 実装改変はしない（docs/運用SOT固定のみ）

## Hard Rules (Project)
- **exit 系全面禁止**（shell: exit/return/set -e/trap EXIT 等、Python: sys.exit/SystemExit/assert 等）
- 失敗は「ERROR: ...」を出して **STOP=1** にし、以降フェーズを SKIP（理由1行）
- すべてのフェーズは **止まらない**（落ちない）設計：コマンド失敗は `|| true`
- 判定は終了コードではなく **出力テキスト/文字列/空非空**で行う
- 重い処理は分割。CPU が跳ねそうなら即座に SKIP（理由を1行残す）

## Edited Files (Fixed Real Paths)
- docs/ops/S12-08_PLAN.md
- docs/ops/S12-08_TASK.md
- docs/ops/STATUS.md
- docs/pr/PR-92-s12-07-guard-sot-docnames-stdout-audit.md (new)

## Inputs (Known Evidence)
- PR: #92
- Merge commit: 33ab2bd
- Evidence (local):
  - prverify_20260224T073824Z_7362237.md
  - veil-rs_review_strict_20260224_073756_736223723565.tar.gz

---

## Pseudocode (Stopless)

### Global
- STOP := 0
- ROOT := git repo toplevel (best-effort)
- TS := utc timestamp
- OBS := .local/obs/s12-08_sot_closeout_${TS}

### PHASE 0: discovery (light)
try:
  - locate ROOT
  - create OBS dir
  - capture:
    - STATUS.md row for S12-07 (raw line)
    - docs/pr directory listing
    - (optional) detect existing PR-92 doc name collision
catch:
  - print "ERROR: discovery_failed reason=..."
  - STOP=1

if STOP==1:
  print "SKIP: phase=discovery reason=stop=1"
else:
  print "OK: phase=discovery"

### PHASE 1: create S12-08 PLAN/TASK (light)
if STOP==1:
  print "SKIP: phase=plan_task reason=stop=1"
else:
  write:
    - docs/ops/S12-08_PLAN.md (this file)
    - docs/ops/S12-08_TASK.md
  print "OK: phase=plan_task"

### PHASE 2: create PR-92 doc (light)
if STOP==1:
  print "SKIP: phase=pr_doc reason=stop=1"
else:
  write:
    - docs/pr/PR-92-s12-07-guard-sot-docnames-stdout-audit.md
      include: summary, known evidence strings, merge commit 33ab2bd
  print "OK: phase=pr_doc"

### PHASE 3: update STATUS (light)
if STOP==1:
  print "SKIP: phase=status_update reason=stop=1"
else:
  goal:
    - S12-07 row => "100% (Merged PR #92)" and Evidence => docs/pr/PR-92-...md
    - S12-08 row => "1% (WIP)" and Evidence => docs/ops/S12-08_PLAN.md (placeholder禁止回避)
  method:
    - try patch by parsing markdown table row "| S12-07 | ... |"
    - if patch fails, print ERROR and set STOP=1 (do not lie)
  print "OK: phase=status_update" OR "ERROR: phase=status_update ..."

### PHASE 4: verify minimal (light)
if STOP==1:
  print "SKIP: phase=verify reason=stop=1"
else:
  - rg で参照整合チェック（S12-07, PR-92 doc path, 33ab2bd）
  - git diff で差分確認（docsのみ）
  - heavy verify (nix/prverify) は **SKIP**（理由1行）
  print "OK: phase=verify"

### PHASE 5: PR create (light)
if STOP==1:
  print "SKIP: phase=pr_create reason=stop=1"
else:
  - commit (docs only)
  - push
  - gh pr create
  print "OK: phase=pr_create"

PHASE END:
print "OK: phase=end stop=<0|1>"

---

## Acceptance (DoD)
- docs/pr/PR-92-...md が存在し、S12-07 の成果と evidence が固定されている
- STATUS.md 上で S12-07 が 100% (Merged PR #92) になり、Evidence が docs/pr を指す
- STATUS.md 上で S12-08 が 1% (WIP) になり、Evidence が docs/ops/S12-08_PLAN.md を指す
- 実行ログ（OBS）に discovery の観測結果が残る
- すべて stopless（ERROR は STOP=1、以降は SKIP 理由1行）

## Progress
- S12-07: 100% (Merged PR #92 / merge commit 33ab2bd)
- S12-08: 0% -> (this PR) 1% (WIP) を STATUS に刻む
