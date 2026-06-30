# 13. Implementation Roadmap

## Phase 0: SOT Freeze

- [x] Local-first / Evidence / Verify / GTM SOT確定
- [x] check_gtmで禁止語・版ブレ防止
- [ ] この設計書を `docs/design/enterprise_jp_pii/` へ格納
- [ ] `.gitignore` に `.codex/`, `.design/`, `.private/` があることを確認

## Phase 0.5: Contract Alignment PR

- [ ] Rust DTOを `crates/veil-pro/src/api/dto.rs` に集約
- [ ] OpenAPI/JSON Schema生成タスクを追加
- [ ] Evidence ZIP baseline entryを `veil.baseline.json` に統一
- [ ] score/grade/severity mapping testを追加
- [ ] skip/incomplete分類 testを追加
- [ ] LSP span/range internal modelを追加
- [ ] `scripts/check_generated_schemas.py` 名称に統一
- [ ] schema出力先を repo root `schemas/` に固定
- [ ] `EvidenceSummary.suppressedSeverityCounts` を必須化
- [ ] `RunMetaResponse` を full RunMetaV1 として実装
- [ ] `coverageComplete` を API/report/run_meta に追加
- [ ] `json-schema.report.json` に `$defs.SafeFindingApiV1` を埋め込む
- [ ] `14_bulk_implementation_safety.md` のAcceptance GateをCIへ追加

## Phase 1: JP-PII Engine Hardening

- [ ] `jp_normalize` 実装
- [ ] span mapping導入
- [ ] MyNumber validator / Luhn validator
- [ ] `fintech-jp`, `gov-jp`, `logs-jp` presets追加
- [ ] Positive/Negative fixtures追加
- [ ] score調整

PR分割:
1. `feat(core): add jp normalization and span mapping`
2. `feat(rules): add fintech-jp/gov-jp/logs-jp presets`
3. `test(jp-pii): add positive and negative fixtures`

## Phase 2: Zero-Config & Preset UX

- [ ] `CoreConfig.preset` 追加
- [ ] `veil init --preset` 実装
- [ ] `veil scan --preset` 実装
- [ ] wizard推論ロジック
- [ ] docs/README更新

## Phase 3: Interactive CLI

- [ ] `veil scan --interactive` flag
- [ ] Finding iteration state machine
- [ ] terminal rendering
- [ ] diff preview
- [ ] atomic write
- [ ] tests with scripted stdin

## Phase 4: LSP

- [ ] `crates/veil-lsp` 作成
- [ ] tower-lsp integration
- [ ] diagnostics mapping
- [ ] code actions
- [ ] Neovim doc

## Phase 5: Local Audit UI completion

- [ ] API schema alignment
- [ ] Svelte state machine
- [ ] Policy explain
- [ ] Evidence ZIP UX
- [ ] PDF export path検討

## Phase 6: PDF Report

### 方針
HTMLをSOTにし、PDFは印刷最適CSSまたはローカルheadless rendererを別optional featureとする。

- [ ] `report.print.html` CSS
- [ ] `Export PDF` はbrowser printを初期案にする
- [ ] 将来: `--format pdf` はfeature gated

## Phase 7: Enterprise hardening

- [ ] Offline RulePack signature verification
- [ ] RulePack update flow
- [ ] Enterprise policy templates
- [ ] Evidence signing playbook

## 完了条件

- `veil scan --preset fintech-jp` がZero-Configで実行可能
- `veil scan --interactive` がatomic write preview付きで動作
- `veil lsp` がNeovimでdiagnostics表示
- `veil ui` がEvidence ZIPを出力
- `veil verify evidence.zip` が検証可能
- JP-PII fixtureで検知率/誤検知率が契約化されている


## Phase 0.6: Bulk Implementation Safety Gate

全部まとめて実装する場合でも、内部順序は以下に固定する。

```text
DTO/schema生成 → Core model移行 → Evidence契約 → CLI exit契約 → Local API → UI → LSP
```

各層のrollback条件、feature/default方針、acceptance gateは `14_bulk_implementation_safety.md` をSOTとする。


## Contract polish acceptance tasks

- [ ] `coverageComplete` を Local API DTO / OpenAPI / JSON Schema / examples で必須化。
- [ ] `SeverityCounts` を zero-filled 4-key object に固定。
- [ ] legacy `severity` migration tableを実装。
- [ ] acceptance gate を `cargo run -p veil-cli -- verify ...` に固定。
- [ ] Evidence `report.json` が raw-free all findings を含むことをfixtureで検証。
