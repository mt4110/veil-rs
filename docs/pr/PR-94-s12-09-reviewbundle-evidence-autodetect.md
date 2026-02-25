# PR-94: S12-09 reviewbundle evidence auto-discovery + strict UX hardening

## Reference
- PR: https://github.com/mt4110/veil-rs/pull/94
- SOT: S12-09
- Phase: Review (99%)

## Title
S12-09 reviewbundle evidence auto-discovery + strict UX hardening

## Body
`reviewbundle create --mode strict` における evidence 解決を 自動化 + 堅牢化しました。
重い `prverify` の自動実行は行わず、明示指定 (`--evidence-report`) または 自動探索 により「嘘のない evidence 束縛」を実現します。

### Key Changes
- **Explicit Flag**: `--evidence-report <path>` を追加（最優先で採用）。
  - 指定されたエビデンスが無効（読取失敗・SHA不一致）な場合はフォールバックせず即座に `stop=1` となるよう剛性を強化。
- **Auto-Discovery (CPU-safe)**:
  - 探索先: `.local/prverify/` / `docs/evidence/prverify/`（トップレベルのみ・ReadDir 前提）
  - HEAD SHA (12-char prefix) を filename/content の両面でマッチ。
  - 選定結果を `OK: evidence_candidate=...` で固定（監査ログ）。
- **Strict Hardening (truthful stopless)**:
  - evidence が確定できない場合は strict tarball を生成しない。
  - `OK: phase=end stop=1` を出し、stdout contract で停止を表明。プロセス終了（`os.Exit`）を伴わない安全な一本道を完成させました。
- **WIP Mode Compatibility**: WIPモードでは引き続き evidence 任意とし、既存の柔軟性を維持。

### Evidence / SOT
- [PLAN](../ops/S12-09_PLAN.md)
- [TASK](../ops/S12-09_TASK.md)
- [STATUS](../ops/STATUS.md) (S12-09: 99% Review)

### Verification Result
- Targeted Go test: `GOMAXPROCS=2 go test ./cmd/reviewbundle -p 1 -count=1`
- Manual walkthrough: [walkthrough.md](../../.gemini/antigravity/brain/1b5c46bf-08dd-46e3-a895-5e81eac98252/walkthrough.md)（happy path: `ccdf11d` の実観測ログ + 監査結果）

### Commit
`ccdf11d` (Verified Happy Path with Evidence)
