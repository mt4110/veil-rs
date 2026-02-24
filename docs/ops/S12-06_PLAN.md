# S12-06 PLAN — Three Themes (B→C→A 推奨順)

**Status**: 0% (Kickoff前 / テーマ選択待ち)
**推奨実施順**: B → C → A（下記理由参照）

---

## なぜ B→C→A か

| 順序 | テーマ                    | 理由                                                             |
| ---- | ------------------------- | ---------------------------------------------------------------- |
| ① B  | pack contract を法律化    | "壊れない土台"を先に定義。Aのchain testの前提になる              |
| ② C  | .local GC（運用破綻根絶） | CPUもディスクも守る。軽い。Bの実装の後で邪魔にならない           |
| ③ A  | verify chain hardening    | B/Cが済んだ後に連鎖全体を整流する（最重量だが、Bがないと空振り） |

---

## B案: S12-06B — pack contract を"法律"にする

**Branch**: `s12-06-pack-contract-law-v1`

**Goal**: review bundle / review pack の manifest / schema / 互換を仕様固定し、破壊的変更をテストで止める。

**DoD**:
1. Contract v1 ドキュメントが存在 (何が保証で何が非保証か明文化)
2. `create` が `contract_version` と manifest を必ず決定論的に出す
3. `verify` が contract を検査し、違反は `ERROR:` + `stop=1` (os.Exit禁止)
4. 互換テスト（旧bundle→新verify）が追加される

### Pseudocode

```
PHASE 0: discovery (軽い)
  rg: cmd/reviewbundle の現状 contract / manifest 出力を棚卸し
  IF missing paths THEN ERROR + STOP=1
  ELSE OK

PHASE 1: define contract (軽い)
  CREATE docs/ops/REVIEW_BUNDLE_CONTRACT_v1.md:
    - contract_version フィールド定義 (唯一の真実 = manifest のみ)
    - required files 一覧
    - required manifest fields
    - normalization rules (ordering, timestamp policy)
    - forward/backward compatibility policy (semver-like)
  NOTE: contract.json は将来増えるなら作る (現時点不要)
        manifest が contract_version の「唯一の真実」

PHASE 2: implement manifest emission (中)
  IF STOP=1 THEN SKIP
  UPDATE cmd/reviewbundle/create.go:
    - emit manifest deterministically
    - include contract_version IN MANIFEST ONLY (唯一化)

PHASE 3: implement contract verifier (中)
  IF STOP=1 THEN SKIP
  ADD verifier (Go) that checks contract v1 requirements:
    - reads contract_version from manifest (唯一の真実)
    - on violation: fmt.Println("ERROR: contract_violated ...")
    - STOP=1, no os.Exit, no panic, no log.Fatal, no log.Panic
    - always: fmt.Printf("OK: phase=end stop=%d\n", stopVal)

PHASE 4: compatibility tests (中)
  priority order:
    1. unit: manifest schema invariants (必須)
    2. testdata minimal (manifest JSONのみ, tar.gz禁止)
    3. golden bundle: generate in test deterministically (巨大化禁止)
  tar.gz を repo に入れない (差分ノイズ源を避ける)

PHASE 5: STATUS + PR doc (軽い)
  update STATUS.md: S12-06 → 99% (Review)
  create docs/pr/PR-XX-s12-06-pack-contract-law.md

ALWAYS:
  fmt.Printf("OK: phase=end stop=%d\n", stopVal)
```

### 主な編集対象ファイル

- `cmd/reviewbundle/create.go` — manifest/contract_version 追加
- `cmd/reviewbundle/verify.go` or `verify_contract.go` — verifier追加
- `cmd/reviewbundle/*_test.go` — 互換テスト追加
- `docs/ops/REVIEW_BUNDLE_CONTRACT_v1.md` — 新規
- `docs/ops/S12-06_PLAN.md` / `S12-06_TASK.md` / `STATUS.md` / `docs/pr/PR-XX-*.md`

---

## C案: S12-06C — .local を"散らからない宇宙"にする

**Branch**: `s12-06-local-durability-gc-v1`

**Goal**: .local の肥大化・ゴミ化で「再現不能/ディスク死」が起きる未来を先に潰す。削除は慎重に：dry-run がデフォ。

**DoD**:
1. `.local` の保存/保持/掃除ポリシーが docs にある
2. 棚卸しコマンドがあり、出力が `OK:` / `SKIP:` / `ERROR:` で整う
3. GCツール（Go）が `dry-run→plan→apply` の段階制で動く
4. `apply` は明示フラグなしで実行されない（安全第一）

### Pseudocode

```
PHASE 0: inventory (軽い / fast path 必須)
  fast inventory only:
    list .local subdirs + newest mtime + file count (サイズ深掘り無し)
  output OK: dir=<dir> count=<N> newest=<ts>
  decide candidates for retention policy

PHASE 1: define policy (軽い)
  CREATE docs/ops/LOCAL_STORAGE_POLICY_v1.md:
    - what to keep (永続)
    - what to archive (tarball化 → 移動)
    - what to delete (期限切れ or 明示指定)
    - default: "never delete without explicit flag"

PHASE 2: implement gc tool (中)
  CREATE cmd/localgc/main.go:     # ハイフン無し (import名衝突回避)
    --mode dry-run (default): 削除候補を列挙のみ (何も消さない)
    --mode plan:   上位ディレクトリ単位でサイズ計測 + 理由付き一覧
    --mode apply + --apply: 両フラグ同時必須 (二重ロック)
      → --mode apply だけでは削除しない
      → --apply だけでは削除しない
    any failure: ERROR + stop=1 (panic/os.Exit/log.Fatal/log.Panic禁止)
    always: OK: phase=end stop=<0|1>

PHASE 3: integrate (軽い)
  mention in ops docs
  CI: dry-run のみ使用可 (apply禁止)

PHASE 4: tests (軽い)
  unit: path selection rules + output format (FS副作用なし)

PHASE 5: STATUS + PR doc (軽い)
```

### 主な編集対象ファイル

- `cmd/localgc/main.go` — 新規 GCツール（ハイフン無し）
- `docs/ops/LOCAL_STORAGE_POLICY_v1.md` — 新規
- `docs/ops/S12-06_PLAN.md` / `S12-06_TASK.md` / `STATUS.md` / `docs/pr/PR-XX-*.md`

---

## A案: S12-06A — verify chain hardening（検証連鎖の完全整流）

**Branch**: `s12-06-verify-chain-hardening-v1`

**Goal**: prverify / reviewbundle / flake / CI の stop解釈を連鎖全体で一貫させ「どこかが嘘をつく可能性」をゼロに近づける。

**DoD**:
1. 全入口/consumer の stop解釈が文書化・統一
2. "exit codeで成功扱い"が起きないテストが存在
3. どの入口も `OK: phase=end stop=<0|1>` を最終的に必ず出す

### Pseudocode

```
PHASE 0: map chain (軽い)
  enumerate all entrypoints: prverify / flake.nix / reviewbundle / CI scripts
  record what each consumer currently parses (stop? exit code? nothing?)
  output as OK: consumer=<name> parses=<stop|exitcode|none>

PHASE 1: unify output contract (中)
  standardize stdout-only machine lines: OK:/ERROR:/SKIP:/PASS:
  stderr: optional detail only
  stop flag: single source of truth (hasError)
  doc: docs/ops/STDOUT_CONTRACT_v1.md

PHASE 2: update consumers (中)
  IF any consumer reads exit code → migrate to stop=1 parse
  (B and S12-05.6 may have already handled most; audit first)

PHASE 3: add chain tests (中)
  integration test (small):
    intentional violation → must yield stop=1 + ERROR: line
    keep hermetic, no real cargo/nix invocations

PHASE 4: STATUS + PR doc (軽い)
```

### 主な編集対象ファイル

- `cmd/prverify/main.go` (追補のみ)
- `flake.nix` (追補のみ)
- `cmd/reviewbundle/create.go` (追補のみ)
- `docs/ops/STDOUT_CONTRACT_v1.md` — 新規
- chain test ファイル (新規)
- `docs/ops/S12-06_PLAN.md` / `S12-06_TASK.md` / `STATUS.md` / `docs/pr/PR-XX-*.md`

---

## 共通 Invariants（全テーマ）

- `os.Exit` / `panic` 禁止（stop=1 + ERROR: で表現）
- stdout = canonical (`OK:` / `ERROR:` / `SKIP:` / `PASS:`)
- 重い処理は分割（端末フリーズ回避）
- `docs/pr/` に PR SOT を置く（check-sot guard 対応）
- `OK: phase=end stop=<0|1>` は必ず最後に1回だけ
