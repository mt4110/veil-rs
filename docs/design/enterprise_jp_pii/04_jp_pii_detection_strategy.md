# 4. JP-PII検知戦略

## 4.1 基本方針

JP-PII検知は **Regex単体ではなく、正規化 + 候補抽出 + validator + context scoring** の4段構成にする。

```text
Raw Line
→ Normalization variants
→ Candidate regex extraction
→ Validator / checksum / context check
→ Score / grade / masked snippet
```

## 4.2 正規化設計

### 対象
| 入力揺れ | 正規化 |
|---|---|
| 全角数字 `１２３` | 半角数字 `123` |
| 全角英字 | 半角英字 |
| 全角スペース | 半角スペース or preserve map |
| ハイフン類 `－―‐‑–—` | `-` |
| 長音符 `ー` | 常に `-` へ畳まない。JP-aware ruleが明示的に許可する場合のみ候補文字として扱う |
| コロン `：` | `:` |
| 丸括弧全角 | 半角 |
| 旧字体/異体字 | v1では扱わず辞書拡張点 |

### NormalizedText
```rust
pub struct NormalizedText {
    pub normalized: String,
    pub index_map: Vec<OriginalSpan>,
}

pub struct OriginalSpan {
    pub normalized_start: usize,
    pub normalized_end: usize,
    pub original_start: usize,
    pub original_end: usize,
}
```

### 必須要件
- Findingの `line_number` / `masked_snippet` は **original** を基準にする。
- 検知は normalized 上で行っても、マスク範囲は `index_map` で original span に戻す。
- `NormalizedText` は original本文を保持しない。originalは呼び出し側が所有し、`index_map` だけでspanを復元する。
- normalized matching は全ルールの既定挙動にしない。JP-aware ruleとして明示されたルールだけが normalized variant を使える。
- LSP rangeも UTF-16位置へ変換する。

## 4.3 マイナンバー検知

### ルール階層
| ルール | 説明 | grade | base_score |
|---|---|---:|---:|
| `pii.jp.mynumber.keyword` | キーワード付き12桁 | High | 92 |
| `pii.jp.mynumber.unlabeled` | ラベルなし12桁 | Medium | 72 |
| `pii.jp.mynumber.strong_context` | 周辺行に本人確認/行政/税/社保 | Critical候補 | 95+ |

### 候補抽出Regex
```regex
(?:\d{4}[- ]?\d{4}[- ]?\d{4})
```

### キーワード
```text
マイナンバー, 個人番号, 基礎マイナンバー, My Number, social security, 税, 扶養, 年末調整, 社保
```

### Validator
- 数字以外を除去して12桁か確認。
- v1ではJ-LISの完全チェックデジットは feature flag `jp_mynumber_checksum` として追加可能にする。
- ラベルなし12桁は単独でHighにしない。
- 同一行に `test`, `dummy`, `sample`, `example`, `0000` 反復がある場合は scoreを減衰。

## 4.4 氏名検知

### 方針
氏名は誤検知が多いため **ラベル付き + 文字種 + 長さ + context** でのみ検知。

### Regex案
```regex
(?:氏名|名前|お名前|Name|name)[^\n]{0,6}[ 　]*[:：]?[ 　]*[A-Za-zァ-ヺぁ-ゖ一-龯][A-Za-zァ-ヺぁ-ゖ一-龯 　]{1,40}
```

### 抑制条件
- `class Name`, `function name`, `name = "test"` など開発語彙は減衰。
- ラベルがなければ検知しない。
- `山田太郎` 等の裸文字列検知はEnterprise RulePack扱い。

## 4.5 住所検知

### 方針
都道府県 + 市区町村 + 番地を基本とする。

### 検知条件
- 都道府県名がある。
- 40文字以内に `市|区|町|村` がある。
- 数字番地がある。

### 揺れ対応
- `1-2-3`, `1丁目2番3号`, `１丁目２番３号` を正規化。
- `東京都千代田区丸の内1-1-1` のような連結文字列対応。

### Score
| 条件 | 加点 |
|---|---:|
| 都道府県 | +20 |
| 市区町村 | +15 |
| 番地 | +15 |
| 郵便番号同一行/前後行 | +15 |
| キーワード `住所` | +15 |

## 4.6 電話番号

### Mobile
```regex
0[789]0[- ]?\d{4}[- ]?\d{4}
```

### Landline
- キーワードなしは誤検知が多いためデフォルトLow/disabled候補。
- `電話|TEL|Phone` 付きで検知。

## 4.7 クレジットカード

### 候補抽出
- Visa/Mastercard/JCB/AMEX/Diners/Discoverに対応。
- キーワード付きはHigh。
- キーワードなしは Luhn validator を必須化。

### Validator
```rust
fn luhn_check(digits: &str) -> bool;
```

### 抑制
- `4111111111111111` などテスト番号は `test_card` tagを付与し、文脈により `Safe`（通常出力しない）または `Low` へ減衰する。`Safe` はv1のgrade/severityには存在しないため使わない。
- `example`, `dummy`, `sandbox` contextで減衰。

## 4.8 銀行口座

### 方針
- `口座番号`, `account number`, `acct_no` などラベル必須。
- 6〜8桁。
- 銀行名/支店名/名義人が近接する場合に加点。

## 4.9 ログファイル対応

### ログ特有の課題
- JSONログ, nginxログ, applicationログにPIIが混在。
- 1行が長い。
- 同一フィールド名が繰り返される。

### 設計
- `.log`, `.jsonl`, `.ndjson` は `logs-jp` presetを推奨。
- JSON Linesは1行scanを維持しつつ、key名をcontextに使う。
- 例: `"phone":"090..."` は `phone` contextで加点。

## 4.10 ルール定義例

```toml
[[rules]]
id = "pii.jp.mynumber.keyword"
description = "マイナンバー（キーワード付き）"
pattern = "(?:(?:基礎)?マイナンバー|個人番号|My\\s*Number)[^0-9０-９]{0,24}[0-9０-９]{4}[- 　－ー]?[0-9０-９]{4}[- 　－ー]?[0-9０-９]{4}"
base_score = 92
category = "pii"
tags = ["jp", "mynumber", "government"]
validator = "jp_mynumber_len12"
context_lines_before = 1
context_lines_after = 1
```

`ー` はこの例のようにルール側で明示した場合にだけ候補として扱う。汎用正規化層で常に `-` へ畳むと、日本語本文の意味を壊し、original span mappingの説明責任も弱くなる。

## 4.12 Score / Severity確定規則

JP-PII rule は Rule定義の `base_score` から開始し、validator/context/negative contextでFindingの最終 `score` を決める。Rule定義では `score` を使わない。最終公開severityは rule定義のseverityではなく、最終scoreから導出する。

| score | grade/severity | 例 |
|---:|---|---|
| 0-19 | Safe（通常出力しない） | dummy/example |
| 20-39 | Low | 弱いラベル、低信頼 |
| 40-69 | Medium | ラベル付きだがvalidator弱 |
| 70-89 | High | validator通過、強文脈 |
| 90-100 | Critical | MyNumber/カード等の強文脈 + validator |

Fail条件は `score` を正本にする。`--fail-on-severity High` は `score >= 70` と同義。

## 4.13 実装タスク

- [x] `jp_normalize.rs` を追加。
- [x] `NormalizedText` と normalized span -> original byte span mapping を実装。
- [x] Ruleに `validator_id` を追加し、TOML `validator` を allowlist から解決。
- [x] `jp_mynumber_len12`, `jp_phone_mobile`, `luhn` validators を実装。
- [x] MyNumber / mobile phone / credit-card positive fixturesを `tests/fixtures/jp_pii/` に追加。
- [x] order number / version number / dummy-test-example / fullwidth non-JP secret / known test card negative fixturesを追加。
- [x] `standard-jp`, `fintech-jp`, `gov-jp`, `si-vendor-jp`, `logs-jp` preset override resolverを追加。
- [x] `veil scan --preset`, `veil init --preset`, `veil config dump --preset` CLI UXを追加。
- [ ] Address validatorを実装する。現状の `pii.jp.address.prefecture_heuristic` はvalidatorなしのヒューリスティックであり、実装済みvalidatorとは扱わない。
- [ ] Name validatorを実装する。現状の `pii.person.name.keyword` はラベル付きヒューリスティックであり、実装済みvalidatorとは扱わない。
- [ ] J-LIS MyNumberチェックデジットを feature flag `jp_mynumber_checksum` として後続実装する。
