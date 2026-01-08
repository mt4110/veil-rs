# veil scan

`veil scan` は、ファイルシステムやGit履歴からシークレットを検出するためのメインコマンドです。

## Usage

```bash
veil scan [OPTIONS] [PATHS]...
```

### Options

| Option | Description |
| :--- | :--- |
| `[PATHS]...` | スキャン対象のパス。デフォルトはカレントディレクトリ (`.`)。 |
| `--staged` | Gitのステージングエリアにあるファイルのみスキャンします。 (`pre-commit` 用) |
| `--since <TIME>` | 指定した時間以降のGit変更履歴のみスキャンします。 (例: `1 week ago`, `2024-01-01`) |
| `--commit <SHA>` | 特定のコミットのみスキャンします。 |
| `--format <FORMAT>` | 出力フォーマット (`text`, `json`, `html`, `markdown`, `table`)。デフォルトは `text`。 |
| `--fail-score <SCORE>` | 指定したスコア以上の検出があった場合、終了コード 1 で終了します。 |
| `--fail-on-severity <LEVEL>` | 指定した重要度以上の検出があった場合、終了コード 1 で終了します。 |

### Examples

#### 1. 基本スキャン
カレントディレクトリ以下を再帰的にスキャンします。
```bash
veil scan .
```

#### 2. JSON出力 (連携用)
検出結果をJSON形式で出力します。
```bash
veil scan . --format json > report.json
```

#### 3. HTMLレポート生成
ブラウザで閲覧可能なリッチなレポートを生成します。
```bash
veil scan . --format html > report.html
open report.html
```

#### 4. Staged Scan (コミット前チェック)
コミットしようとしているファイルだけをチェックします。高速です。
```bash
veil scan --staged
```

#### 5. CIでの利用 (Fail設定)
スコア80以上の検出があった場合にCIを失敗させます。
```bash
veil scan . --fail-score 80
```
