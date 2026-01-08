# veil init

`veil init` は、Veil の設定ファイルを対話的、またはスクリプトで初期化するためのコマンドです。

## Usage

```bash
veil init [OPTIONS]
```

### Options

| Option | Description |
| :--- | :--- |
| `--wizard` | 対話モードで設定作成ウィザードを起動します。 (推奨) |
| `--profile <PROFILE>` | 事前定義されたプロファイルで初期化します。 (`Application`, `Library`, `Logs`) |
| `--ci <PROVIDER>` | CI用の設定ファイルを生成します。 (現在は `github` のみサポート) |
| `--non-interactive` | 非対話モード。ファイルが既に存在する場合はエラーになります。 |

### Examples

#### 1. 対話的ウィザード (推奨)
プロジェクトに合わせた最適な設定を行うためのウィザードを起動します。
```bash
veil init --wizard
```
> **Note:** ウィザード内で「標準ルール (Built-in Rules) を有効にしますか？」と聞かれます。通常は **Yes** を選択してください。これで標準的なPIIやシークレット検出ルールが適用されます。

#### 2. プロファイルを指定して初期化
特定のユースケースに合わせて、推奨設定で即座に初期化します。

- **Application**: 標準的なセキュリティ設定。バランス重視。
- **Library**: 厳格なコンプライアンス設定。外部公開用などに。
- **Logs**: ログ秘匿専用。ソースコードは対象外。

```bash
# ログ秘匿用の設定を作成 (例: ログサーバー用)
veil init --profile Logs
```

#### 3. CI設定の生成
GitHub Actions 用のワークフローファイルを生成します。
```bash
veil init --ci github
```
