# veil guardian

`veil guardian` は、依存関係ロックファイル (`Cargo.lock`, `package-lock.json` 等) を解析し、既知の脆弱性を検出する機能です。
OSV (Open Source Vulnerability) データベースなどを利用します。

## Usage

```bash
veil guardian [COMMAND]
```

### Commands

- `check`: ロックファイルをチェックします。

### Options (for `check`)

| Option | Description |
| :--- | :--- |
| `lockfile` | ロックファイルのパス。デフォルトは `Cargo.lock`。 |
| `--osv-details` | OSVから詳細情報を取得・表示します (ネットワーク接続が必要な場合があります)。 |
| `--offline` | オフラインモード。キャッシュのみを使用します。 |
| `--format <FORMAT>` | 出力フォーマット (`human`, `json`)。 |

### Examples

#### 1. 通常スキャン (高速)
```bash
veil guardian check
```

#### 2. 詳細表示
脆弱性の詳細情報 (CVE ID, 重要度, 修正バージョン等) を含めて表示します。
初回はデータをダウンロードするため時間がかかる場合があります。
```bash
veil guardian check --osv-details
```

#### 3. オフラインモード
ダウンロード済みのキャッシュのみを使用してチェックします。
```bash
veil guardian check --osv-details --offline
```

#### 4. forced Update
CI等で強制的に最新の脆弱性情報を取得したい場合は、環境変数をセットします。
```bash
VEIL_OSV_FORCE_REFRESH=1 veil guardian check --osv-details
```
