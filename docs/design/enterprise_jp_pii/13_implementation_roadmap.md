# 13. Implementation Roadmap

## Phase 0: SOT Freeze

- [x] Local-first / Evidence / Verify / GTM SOT確定
- [x] check_gtmで禁止語・版ブレ防止
- [x] この設計書を `docs/design/enterprise_jp_pii/` へ格納
- [x] `.gitignore` に `.codex/`, `.design/`, `.private/` があることを確認

## Phase 0.5: Contract Alignment PR

- [x] Rust DTOを `crates/veil-pro/src/api/dto.rs` に集約
- [x] OpenAPI/JSON Schema生成タスクを追加
- [x] Evidence ZIP baseline entryを `veil.baseline.json` に統一
- [x] score/grade/severity mapping testを追加
- [x] skip/incomplete分類 testを追加
- [x] LSP span/range internal modelを追加
- [x] `scripts/check_generated_schemas.py` 名称に統一
- [x] schema出力先を repo root `schemas/` に固定
- [x] `EvidenceSummary.suppressedSeverityCounts` を必須化
- [x] `RunMetaResponse` を full RunMetaV1 として実装
- [x] `coverageComplete` を API/report/run_meta に追加
- [x] `json-schema.report.json` に `$defs.SafeFindingApiV1` を埋め込む
- [x] `14_bulk_implementation_safety.md` のAcceptance GateをCIへ追加

## Phase 1: JP-PII Engine Hardening

- [x] `jp_normalize` 実装
- [x] span mapping導入
- [x] MyNumber validator / Luhn validator / mobile validator / address validator / name validator
- [x] J-LIS MyNumber checksum feature flag
- [x] `standard-jp`, `fintech-jp`, `gov-jp`, `si-vendor-jp`, `logs-jp` presets追加
- [x] Positive/Negative fixtures追加
- [x] score調整（negative context dampeningをJPテストデータ文脈へ拡張）

PR分割:
1. #101 `feat(core): add jp normalization and span mapping`
2. #102 `test(jp-pii): add validators and fixtures`
3. #103 `feat(config): add JP preset override resolver`
4. #112 `Wire JP address validator into address rule`
5. #113 `Tune JP PII negative test context scoring`

## Phase 2: Zero-Config & Preset UX

- [ ] `CoreConfig.preset` 追加（急がない。#106時点ではCLI/Local API引数のbase layer適用を正本とする）
- [x] `veil init --preset` 実装
- [x] `veil scan --preset` 実装
- [x] wizard推論ロジック（候補検出と表示。自動適用はしない）
- [x] docs/README更新
- [x] `veil config dump --preset` 実装
- [x] Local API / UI の preset対応（#106: Local API scan preset解決 + 最小UI selector。大きなUI workflow改修はPhase 5）

## Phase 3: Interactive CLI

- [x] `veil scan --interactive` flag（guarded stub）
- [x] Finding iteration state machine（renderer 未接続）
- [x] terminal rendering（safe masked context）
- [x] diff preview（safe redacted preview）
- [x] atomic write
- [x] tests with scripted stdin

## Phase 4: LSP

- [x] `crates/veil-lsp` 作成
- [x] tower-lsp integration
- [ ] diagnostics mapping
- [ ] code actions
- [ ] Neovim doc

## Phase 5: Local Audit UI completion

- [ ] API schema alignment
- [x] Svelte state machine
- [x] Policy explain
- [x] Evidence ZIP UX
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

- [x] `coverageComplete` を Local API DTO / OpenAPI / JSON Schema / examples で必須化。
- [x] `SeverityCounts` を zero-filled 4-key object に固定。
- [x] legacy `severity` migration tableを実装。
- [x] acceptance gate を `cargo run -p veil-cli -- verify ...` に固定。
- [x] Evidence `report.json` が raw-free all findings を含むことをfixtureで検証。
