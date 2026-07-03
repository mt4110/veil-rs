# JP Security Templates 1000

日本語中心のセキュリティリスク・PII・機密情報検知テンプレート集です。
既存の `[[rules]]` TOML構造に寄せて、**1ファイル1ルール** で生成しています。

このディレクトリは **テンプレート置き場** です。`rules_ja/default` の実行RulePackには直接入れず、必要なカテゴリ・variantだけを選別して採用します。

## 内容

- TOMLテンプレート数: 1000
- コンセプト数: 250
- 1コンセプトあたり4種類:
  - `lv`: 日本語ラベル付き値
  - `kv`: JSON/YAML/env風キー値
  - `schema`: CSV/DBスキーマ上の機微カラム名
  - `leak`: ログ・通知・外部送信に出る漏えい兆候

## ディレクトリ構造

```text
templates/<category>/<variant>/log.jp.<category>.<slug>.<variant>.toml
```

例:

```text
templates/finance/kv/log.jp.finance.bank_account.kv.toml
templates/pii/lv/log.jp.pii.full_name.lv.toml
templates/secret/leak/log.jp.secret.api_token.leak.toml
```

`MANIFEST.csv` は全テンプレートの正本indexです。`path`, `id`, `concept`, `slug`, `variant`, `category`, `severity`, `score`, `terms` を持ちます。

## カテゴリ別件数

- `education`: 60
- `finance`: 140
- `government`: 100
- `healthcare`: 100
- `hr`: 100
- `legal`: 80
- `pii`: 240
- `secret`: 120
- `security`: 60

## 重要度別件数

- `critical`: 316
- `high`: 448
- `low`: 16
- `medium`: 220

## 使い方の想定

まずは全部を有効化せず、以下のように段階導入してください。

1. `secret`, `government`, `finance`, `healthcare` の `lv` / `kv` から有効化
2. false positive を見ながら `schema` をCI警告扱いで追加
3. `leak` は監査・ログレビュー用として低めのブロック条件にする

採用時は `veil rules promote-templates` で `category` / `variant` / `severity` / `score` を絞り込み、実行RulePackを生成してください。テンプレート集全体をRulePackとして直接読み込まないでください。

## 注意

- これは初期テンプレートです。日本語ログ・CSV・DBスキーマ・問い合わせ本文などを広く拾うため、意図的に保守的なものと広めのものが混在しています。
- `schema` / `leak` は「値そのもの」より「危険な扱い方・カラム名」を拾うため、マスキングだけでなくレビュー警告にも向いています。
- production RulePackへ昇格する前に、`base_score` を正本にしたscore/severity整理、false positive fixture、validator有無の確認を行ってください。
- すべて `tomllib` でTOML妥当性を検証済みです。
