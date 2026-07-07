# Implementation Task Breakdown

## Contract Alignment（実装済み）

- [x] `crates/veil-pro/src/api/dto.rs` を追加し、Local API DTOを集約する。
- [x] DTOに `serde`, `schemars`, `utoipa` deriveを付ける。
- [x] `crates/veil-pro/src/bin/export_local_api_schema.rs` を追加し、OpenAPI / JSON Schema を repo root `schemas/` へ生成する。
- [x] `scripts/check_generated_schemas.py` を追加し、生成schemaとtracked schemaの差分を検出可能にする。
- [x] `scripts/check_generated_schemas.py` または `scripts/check_contract_acceptance.py` をCI workflowへ配線する。
- [x] `schemas/json-schema.report.json` を生成物として追加し、`$defs.SafeFindingApiV1` を内包する。
- [x] `schemas/json-schema.run-meta.json` から `artifacts.runMeta` を排除する。
- [x] `ScanRequest.paths` missing/empty -> `["."]` 正規化test。
- [x] Evidence ZIP baseline entryを `veil.baseline.json` に統一し、schema/verifyで `path const` を検査する。
- [x] `EvidenceSummary.suppressedSeverityCounts` を API/report/run_meta すべてで必須化する。
- [x] `--fail-on-findings N` を `effectiveFindings >= N` に固定するtest。
- [x] `failOnFindings=0` の CLI Exit 2 / Local API 400 test。
- [x] Rule schemaを `base_score` に統一するmigration/test。
- [x] preset TOML の `severity` を全廃し `base_score` にする。
- [x] LSP `Range { start, end }` UTF-16変換test。
- [x] `RunMetaResponse` が full RunMetaV1 と一致するtest。
- [x] `max_findings` 到達時の `coverageComplete=false` / status incomplete / Exit 2 test。
- [x] `.gitignore` に `.codex/` を追加する。

## #101 JP正規化基盤（merge済み）

- [x] `crates/veil-core/src/scanner/jp_normalize.rs`
- [x] 全角数字/英字/記号/スペース正規化
- [x] normalized span -> original byte span mapping
- [x] UTF-16 Range mapping utility

## #102 JP-PII validators / fixtures（merge済み）

- [x] 既存RulePackの `score` を `base_score` へ移行
- [x] 互換読み込みwarning
- [x] MyNumber validator / Luhn validator / mobile validator
- [x] Positive/Negative fixtures追加
- [x] Address validatorは #112 で実装・配線済み。`pii.jp.address.prefecture_heuristic` は `jp_address_prefecture_city_block` validatorを使う。
- [x] Name validatorは実装・配線済み。`pii.person.name.keyword` は `jp_person_name_keyword` validatorを使う。
- [x] J-LIS MyNumberチェックデジットは default off の feature flag `jp_mynumber_checksum` で実装済み。

## #103 JP preset override resolver（merge済み）

- [x] `standard-jp`, `fintech-jp`, `gov-jp`, `si-vendor-jp`, `logs-jp` builtin preset TOMLを追加。
- [x] preset TOMLを `enabled` / `base_score` だけに制限。
- [x] presetをbase layerとして適用し、repo/org config overrideを維持。
- [x] 新規presetに `severity` を書かない契約をtestで固定。

## #104 JP preset CLI UX（merge済み）

- [x] `veil scan --preset`
- [x] `veil init --preset`
- [x] `veil config dump --preset --layer preset`
- [x] `logs-jp` initで `rules/log` を生成。
- [x] `--ci` と `--preset` / `--profile` / `--wizard` の同時指定を拒否。

## #106 Local API / UI preset support（merge済み）

- [x] Local API scanで `PresetName` を builtin preset configへ解決する。
- [x] `logs-jp` scanではCLIと同じく `rules/log` RulePackを必須にする。
- [x] OpenAPI generated schemaをUI clientへ反映。
- [x] UI scan requestでpresetを渡せる最小導線を確認する。
- [ ] `includeSuppressed` UI toggle
- [ ] limit reached / coverageComplete UI表示

## #112 JP address validator wiring（merge済み）

- [x] `pii.jp.address.prefecture_heuristic` に `jp_address_prefecture_city_block` validatorを配線。
- [x] 全角住所positive fixtureと住所ラベルのみnegative fixtureを追加。
- [x] Built-in ruleでaddress validatorが解決されることをtestで固定。

## #113 JP PII score context tuning（merge済み）

- [x] `sandbox` と日本語のテスト/サンプル/ダミー/モック/サンドボックス文脈でscore減衰する。
- [x] score context modifierの単体テストを追加。
- [x] JP PII fixtureが既存検知挙動を維持することを確認。

## Later: LSP

- [ ] `crates/veil-lsp` 追加
- [ ] Diagnostics / CodeAction / ignore comments
- [ ] JSONでinline ignore actionを出さないtest


## Contract polish sync tasks

- [x] `coverageComplete` を DTO説明・OpenAPI・schema・testで同期。
- [x] `SeverityCounts` を4キー必須に変更。
- [x] legacy severity migration tableをloader testへ追加。
- [x] acceptance gateを `cargo run -p veil-cli -- verify` へ更新。
- [x] Evidence report fixtureに suppressed finding を含める。

- [x] Verify that generated JSON Schema/OpenAPI require `RunMeta.result.limitReasons` and reject unknown result keys.
