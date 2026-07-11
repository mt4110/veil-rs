# veil-rs 国内エンタープライズ向け詳細設計書パック

## 対象
- 国内Fintech / 官公庁 / SIer 向け、完全ローカル実行の JP-PII / Secret 検知・マスキング・監査証跡ツール。
- 現行 `veil-rs` リポジトリを土台に、以下の実装対象を製品版として完成させる。
  - JP-PII 特化エンジン
  - Zero-Config / Preset Templates
  - Interactive CLI
  - LSP Integration
  - Local Audit UI
  - Evidence Pack / verify / 監査レポート

## 収録ドキュメント
| ファイル | 内容 |
|---|---|
| `00_contract_decisions.md` | 実装解釈割れを防ぐ契約決定SOT |
| `01_system_architecture.md` | Mermaid図付きの全体アーキテクチャ |
| `02_component_design.md` | Core / CLI / LSP / UI / Evidence / RulePack の責務とI/O |
| `03_ux_command_design.md` | CLI/UX設計、Interactive CLI状態遷移図 |
| `04_jp_pii_detection_strategy.md` | マイナンバー、住所、氏名、電話、クレカ等の検知戦略 |
| `05_local_audit_ui_api_schema.md` | Local Server ⇔ Svelte APIスキーマ |
| `06_lsp_design.md` | Language Server Protocol 設計 |
| `07_zero_config_and_presets.md` | `veil init` / `--preset` の設計 |
| `08_security_privacy_design.md` | ローカルファースト、CSP、権限、証跡の設計 |
| `09_performance_limits.md` | CI遅延を避ける性能・上限制御設計 |
| `10_evidence_pack_and_verify.md` | 監査証跡ZIPと第三者検証設計 |
| `11_data_model.md` | Finding / Rule / Config / RunMeta のデータモデル |
| `12_testing_strategy.md` | テスト戦略、fixtures、E2E、性能テスト |
| `13_implementation_roadmap.md` | 実装ロードマップ、PR分割、受け入れ条件 |
| `14_bulk_implementation_safety.md` | 一括実装時の順序DAG、feature flag、互換、acceptance gate、rollback条件 |
| `15_contract_confidence_audit.md` | 既知loopholeとv4.4処置の自己監査 |
| `schemas/openapi.local-api.yaml` | Local UI APIのOpenAPI風スキーマ |
| `schemas/json-schema.finding.json` | Finding JSON Schema |
| `schemas/json-schema.safe-finding-api.json` | Local API / Evidence用 SafeFinding JSON Schema |
| `schemas/json-schema.run-meta.json` | RunMeta JSON Schema |
| `schemas/json-schema.report.json` | Evidence report.json JSON Schema |
| `templates/presets/*.toml` | プリセットTOML案 |
| `templates/policies/*.toml` | `VEIL_ORG_CONFIG` 用の企業ポリシーテンプレート |
| `implementation/task_breakdown.md` | 実装タスクのチェックリスト |
| `implementation/risk_register.md` | 技術・UX・営業上のリスク管理 |
| `implementation/evidence_signing_playbook.md` | Evidence ZIPの外部署名・承認運用runbook |
| `implementation/rulepack_update_flow.md` | 署名検証済みRulePackのstaging/promote/rollback運用runbook |

## SOT方針
- 正本優先順は `00_contract_decisions.md` → Rust DTO/Core型 → 生成OpenAPI/JSON Schema → 各章本文。
- 既存の `veil-core`, `veil-cli`, `veil-pro` 実装を活かし、不足機能は差分実装として明記する。
- 外部送信、クラウド集約、0.0.0.0公開は製品思想と矛盾するため、対象外。



## v4.4 contract note

- 実装時は `00_contract_decisions.md` を最上位SOTとし、`PR-0 Contract Alignment` から開始する。
- acceptance gate の唯一の正本は `14_bulk_implementation_safety.md` の `14.4 Acceptance Gate`。
- 実装repoでは本設計パックを `docs/design/enterprise_jp_pii/` に配置する。`.private/` は作業用コピーであり、PR正本にはしない。
- schema正本は repo root `schemas/`。設計パック内 `schemas/` は参照コピー。
- RunMeta `result` は `limitReasons` required かつ `additionalProperties=false`。
- v4.4 は PR-0 Contract Alignment 直前の schema strictness sync 版。
