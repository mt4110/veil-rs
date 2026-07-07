# 12. Testing Strategy

## 12.1 テスト分類

| 種別 | 対象 | 目的 |
|---|---|---|
| Unit | validators, masking, normalization | ロジック正当性 |
| Fixture | JP PII rules | 検知/誤検知の契約 |
| Contract | JSON / Evidence schema | 外部互換性 |
| E2E | CLI / UI / LSP | 実運用導線 |
| Perf | scan / LSP latency | CI遅延防止 |
| Security | CSP / path / ZIP / token | B2B安全契約 |

## 12.2 JP-PII fixtures

```text
tests/fixtures/jp_pii/positive/mynumber_keyword.txt
tests/fixtures/jp_pii/positive/address_fullwidth.txt
tests/fixtures/jp_pii/positive/credit_card_luhn.txt
tests/fixtures/jp_pii/negative/order_numbers.txt
tests/fixtures/jp_pii/negative/dummy_test_cards.txt
tests/fixtures/jp_pii/negative/version_numbers.txt
```

### Positive例
```text
個人番号: １２３４－５６７８－９０１８
東京都千代田区丸の内１丁目１番１号
クレジットカード: 4111 1111 1111 1111
```

### Negative例
```text
build number: 1234-5678-9012
example card: 4111111111111111
version 1.2.3.4
```

## 12.3 LSP E2E

- `didOpen` → diagnostics件数
- `didChange` → debounce後 diagnostics更新
- CodeAction `mask` → workspace edit内容
- UTF-16 range correctness
- masked_snippetからrangeを復元していないこと
- JSONではinline ignore actionが出ないこと

## 12.4 UI E2E

- 起動 → token取得 → `/api/me`
- scan → findings表示
- Evidence export → zip取得
- RunCache TTL切れ → 410 → Re-scan導線
- CSP検査: distに inline script/style/evalなし

## 12.5 Evidence verify tests

- golden zip exit 0
- hash mismatch exit 2
- external anchor mismatch exit 2
- token leak exit 2
- ZipSlip exit 2
- ZipBomb exit 2
- incomplete + require-complete exit 1
- baseline absent exit 0

## 12.6 Performance tests

- Rayon並列のスケール確認
- max_file_count/max_findings/max_file_size(text)到達時のexit 2
- LSP 1k line buffer p95 latency
- normalization span map allocation

## 12.7 Acceptance Gate

本章では acceptance gate の具体コマンドを定義しない。重複定義を避けるため、**唯一の正本は `14_bulk_implementation_safety.md` の `14.4 Acceptance Gate`** とする。

テスト戦略として本章が要求する観点は以下に限る。

- workspace tests が通ること。
- Svelte frontend build が通ること。
- Rust DTO から OpenAPI / JSON Schema を生成できること。
- `scripts/check_generated_schemas.py` が生成物の差分を検出できること。
- Evidence golden ZIP を `cargo run -p veil-cli -- verify ...` で検証できること。

具体コマンド列、PATH依存排除、rollback条件は `14.4` と `14.5` を参照する。

## 12.8 Contract drift tests

- Rust DTO → OpenAPI/JSON Schema生成後、差分がある場合はCIで検出する。
- `paths` 省略/空配列が `["."]` に正規化されること。
- `SafeFindingApiV1` に raw `matchedContent` / `lineContent` が存在しないこと。
- Evidence ZIP の baseline entry が `veil.baseline.json` であること。
- `baseline.json` entry が混入した場合はテストで失敗させる。
- `score → grade/severity` mappingが固定表と一致すること。


## 12.9 Contract Alignment tests

- Rust DTOから生成した OpenAPI / JSON Schema がtracked schemaと一致する。
- `run_meta.json` に `artifacts.runMeta` が存在しない。
- `--expect-run-meta-sha256` はZIP内 `run_meta.json` raw bytesに対して検証する。
- Evidence ZIP内 baseline entry は `veil.baseline.json`。
- Evidence `report.json` は `json-schema.report.json` に適合し、raw-free な全 `SafeFindingApiV1[]` を含む。baseline使用時は `new` と `suppressed` の両方を含む。
- `ScanResponse.totalFindings`, `suppressedFindings`, `effectiveFindings` の意味がbaseline fixtureで固定される。
- `--fail-on-findings 1` は effective finding 1件でExit 1。
- `Rule` schemaに `score` が存在せず、`base_score` だけを持つ。
- LSP `Range { start, end }` がUTF-16位置であることをemoji/全角fixtureで検証する。


## 12.10 v4 Contract Alignment tests

PR-0では以下を必須testにする。

- `scripts/check_generated_schemas.py` が repo root `schemas/` と生成結果の差分を検出する。
- `json-schema.report.json` は `$defs.SafeFindingApiV1` を内包し、外部相対 `$ref` を使わない。
- `run_meta.artifacts.runMeta` が存在しないこと。
- `run_meta.artifacts.baseline.path == "veil.baseline.json"` をschema/verifyで検査すること。
- `EvidenceSummary.suppressedSeverityCounts` が API / report / run_meta すべてで必須であること。
- すべての `SeverityCounts` が `Low` / `Medium` / `High` / `Critical` の4キーを持ち、0件でもキー省略されないこと。
- `max_findings` 到達時は `coverageComplete=false`, status=`incomplete`, Exit 2であること。
- `failOnFindings=0` は CLIでは設定エラー Exit 2、Local APIでは400 `INVALID_REQUEST`。
- Rule定義に `score` / `severity` が残らず、`base_score` のみであること。
- legacy `severity` migration tableが `Low=20`, `Medium=40`, `High=70`, `Critical=90` と一致すること。
- `RunMetaResponse` が full RunMetaV1 と一致すること。
