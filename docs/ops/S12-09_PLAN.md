# S12-09 PLAN: reviewbundle evidence auto-discovery + strict UX hardening (stopless)

## Goal
`reviewbundle create --mode strict` が evidence を必須として扱うとき、
「重い prverify を自動実行せず」確実に evidence を束縛できる設計へ寄せる。

- explicit: `--evidence-report <path>` を提供（探索失敗時の救済）
- auto-detect: 次の場所を探索し、最適候補を選ぶ（軽い）
  - `.local/prverify/`
  - `docs/evidence/prverify/`
- strict rule: evidence が見つからない/読めない場合、**strict bundle は作らない**
- stopless: exit しない（`ERROR:` を出し `stop=1` を宣言して終わる）
- 判定は stdout の `OK/ERROR/SKIP` のみ（終了コードに依存しない）

## Non-Goals
- prverify を自動実行しない（CPU保護 / 循環依存回避）
- wip bundle の契約を過剰に厳格化しない（共有用途として柔らかく維持）

## Hard Rules
- exit 系全面禁止（shell/python/go いずれも「強制終了」設計を持ち込まない）
- 失敗は `ERROR:` を出して STOP=1、以降は SKIP（理由1行）
- 重い処理は 1ステップ1本に分割（固まりそうなら “中断してログだけ残す”）

## Artifacts / Contract
### Evidence selection contract (v1)
Input:
- `--evidence-report <path>` が指定されれば最優先。**読取失敗または SHA不一致（12桁prefix）時は、modeに関わらず即座に stop=1 とし、auto-detect へのフォールバックを禁止する。**
- 未指定なら auto-detect を試す

Auto-detect (light):
1) `.local/prverify/` および `docs/evidence/prverify/` 配下の `prverify_*.md` を列挙
2) まず “HEAD sha をファイル名に含むもの” を候補化（あれば優先）
3) 候補の内容に HEAD sha（12桁prefix）が含まれること（content match）を最終保証とする
4) 同率なら **辞書順で最大**（timestamp 形式を想定）を選ぶ
5) 無ければ “prverify_*.md 全体” から辞書順最大を選ぶ
6) 選定理由を INFO で 1行固定（監査ログ）

Strict behavior:
- evidence が確定できない → `ERROR: evidence_required mode=strict` + `stop=1`
- `stop=1` の場合は **strict tarball を生成しない**（嘘禁止）
- ただし最後に必ず `OK: phase=end stop=1` を出す

WIP behavior:
- evidence は任意（無ければ INFO のみ）
- ある場合は bundle に埋め込む/参照を残す（既存仕様に合わせる）

## Edited Files (Fixed)
- docs/ops/S12-09_PLAN.md
- docs/ops/S12-09_TASK.md
- docs/ops/STATUS.md
- (PR番号確定後) docs/pr/PR-XX-s12-09-reviewbundle-evidence-autodetect.md

## Edited Files (Must be discovered & pinned)
- cmd/reviewbundle/create.go
- cmd/reviewbundle/main.go
- cmd/reviewbundle/consts.go

## Pseudocode (Stopless)

GLOBAL:
  STOP := 0
  OBS := ".local/obs/s12-09_*"

PHASE 0: discovery (light)
  try:
    locate repo root
    write OBS logs
    for each candidate_path in [cmd/reviewbundle, cmd/reviewbundle/*.go, ...]:
      if exists:
        capture ls + rg hits
        break
      else:
        continue
    if not found:
      ERROR + STOP=1
  catch:
    ERROR + STOP=1

PHASE 1: docs scaffold (light)
  if STOP==1:
    SKIP reason=stop=1
  else:
    create PLAN/TASK
    update STATUS: S12-09 = 1% (WIP) Evidence=docs/ops/S12-09_PLAN.md

PHASE 2: implement (light)
  if STOP==1: SKIP
  else:
    add flag: --evidence-report (optional)
    add auto-detect in .local/prverify/ and docs/evidence/prverify/
    strict: evidence missing => ERROR + stop=1 + do not create tarball
    wip: evidence optional

PHASE 3: tests (split)
  if STOP==1: SKIP
  else:
    go test (targeted package only; no go test ./...)
    heavy steps are SKIP unless needed

PHASE 4: minimal verify (split)
  if STOP==1: SKIP
  else:
    case A: evidence present => OK strict bundle created
    case B: evidence missing => ERROR stop=1 and tarball not created
    ensure stdout contains: OK: phase=end stop=<0|1>

PHASE 5: PR (light)
  if STOP==1: SKIP
  else:
    commit/push/PR
    after PR number: create docs/pr/..., update STATUS if needed

END:
  OK: phase=end stop=<0|1>

## DoD
- strict:
  - evidence present => bundle created + evidence path printed
  - evidence missing => ERROR + stop=1 + bundle not created（不生成が証拠）
- wip:
  - evidence optional 維持
- docs:
  - PLAN/TASK/STATUS 更新
  - PR doc 作成（PR番号確定後）
