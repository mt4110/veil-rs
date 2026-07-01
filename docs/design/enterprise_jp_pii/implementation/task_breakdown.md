# Implementation Task Breakdown

## PR-0 Contract Alignment（最初に実装）

- [ ] `crates/veil-pro/src/api/dto.rs` を追加し、Local API DTOを集約する。
- [ ] DTOに `serde`, `schemars`, `utoipa` deriveを付ける。
- [ ] `crates/veil-pro/src/bin/export_local_api_schema.rs` を追加し、OpenAPI / JSON Schema を repo root `schemas/` へ生成する。
- [ ] `scripts/check_generated_schemas.py` を追加し、生成schemaとtracked schemaの差分をCIで検出する。
- [ ] `schemas/json-schema.report.json` を生成物として追加し、`$defs.SafeFindingApiV1` を内包する。
- [ ] `schemas/json-schema.run-meta.json` から `artifacts.runMeta` を排除する。
- [ ] `ScanRequest.paths` missing/empty -> `["."]` 正規化test。
- [ ] Evidence ZIP baseline entryを `veil.baseline.json` に統一し、schema/verifyで `path const` を検査する。
- [ ] `EvidenceSummary.suppressedSeverityCounts` を API/report/run_meta すべてで必須化する。
- [ ] `--fail-on-findings N` を `effectiveFindings >= N` に固定するtest。
- [ ] `failOnFindings=0` の CLI Exit 2 / Local API 400 test。
- [ ] Rule schemaを `base_score` に統一するmigration/test。
- [ ] preset TOML の `severity` を全廃し `base_score` にする。
- [ ] LSP `Range { start, end }` UTF-16変換test。
- [ ] `RunMetaResponse` が full RunMetaV1 と一致するtest。
- [ ] `max_findings` 到達時の `coverageComplete=false` / status incomplete / Exit 2 test。
- [ ] `.gitignore` に `.codex/` を追加する。

## PR-1 JP正規化基盤

- [ ] `crates/veil-core/src/scanner/jp_normalize.rs`
- [ ] 全角数字/英字/記号/スペース正規化
- [ ] normalized span -> original byte span mapping
- [ ] UTF-16 Range mapping utility

## PR-2 JP-PII RulePack migration

- [ ] 既存RulePackの `score` を `base_score` へ移行
- [ ] 互換読み込みwarning
- [ ] MyNumber validator / Luhn validator
- [ ] Positive/Negative fixtures追加

## PR-3 Evidence / Verify contract

- [ ] `report.json` schema validation
- [ ] `run_meta.json` external anchor flow
- [ ] `veil.baseline.json` artifact naming
- [ ] ZipSlip/Bomb/duplicate entry tests

## PR-4 Local API / UI

- [ ] OpenAPI generated schemaをUI clientへ反映
- [ ] `includeSuppressed` UI toggle
- [ ] limit reached / coverageComplete UI表示

## PR-5 LSP

- [ ] `crates/veil-lsp` 追加
- [ ] Diagnostics / CodeAction / ignore comments
- [ ] JSONでinline ignore actionを出さないtest


## v4.4 sync tasks

- [ ] `coverageComplete` を DTO説明・OpenAPI・schema・testで同期。
- [ ] `SeverityCounts` を4キー必須に変更。
- [ ] legacy severity migration tableをloader testへ追加。
- [ ] acceptance gateを `cargo run -p veil-cli -- verify` へ更新。
- [ ] Evidence report fixtureに suppressed finding を含める。

- [ ] Verify generated JSON Schema/OpenAPI require `RunMeta.result.limitReasons` and reject unknown result keys.
