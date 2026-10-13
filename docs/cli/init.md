# veil init

Status: `reference`

スキャン開始に初期化は不要です。設定を保存したい場合にプロジェクトのルートで実行します。

```bash
veil init --non-interactive
veil config check
```

`veil.toml` が既にある場合、非対話モードは終了コード2で保全します。
既定のApplication設定には `core.fail_on_score = 80` と除外等が入り、設定なしのスキャンと失敗条件が変わります。生成内容をレビューしてください。

| オプション | 内容 |
| --- | --- |
| `--non-interactive` | 非対話で生成。既存 `veil.toml` はエラー |
| `--profile Application` | スコア80以上を失敗条件にする通常設定 |
| `--profile Library` | スコア70以上を失敗条件にする設定 |
| `--profile Logs` | `src` を除外し、`rules/log` を使用。スコア失敗条件を設定しないログ用設定 |
| `--wizard` | `wizard` featureを含むビルドの対話設定。既存設定の扱いを選択できる |
| `--ci github` | GitHub Actions設定を生成。既存ワークフローはエラー |
| `--ci-source checkout` | CIでチェックアウトしたリポジトリの `crates/veil-cli` をビルド |
| `--ci-source <COMMIT>` | Veilリポジトリの40桁の完全なコミットSHAからインストール |
| `--pin-tag <TAG>` | 明示した公開タグを使う互換経路。現在の候補の検証を意味しない |

プロファイル名の誤記は現在Applicationへフォールバックします。意図した設定が生成されたかを確認してください。
Logsは単なる出力形式ではなく対象とルールを変える設定です。生成されたルールと除外を確認します。
旧リモートルール設定は新規生成しません。

## CIの取得元を明示する

取得元を省略した `veil init --ci github` はエラーにします。manifestの版から公開タグを推測しません。
既存ワークフローは上書きしません。以下の例は生成先に `.github/workflows/veil.yml` がない場合に使います。
既存ファイルがある場合は保全し、別の作業ディレクトリで生成した内容との差分を確認します。

Veilソースを含むリポジトリのCIでは:

```bash
veil init --ci github --ci-source checkout
```

生成されるCIは、チェックアウトされた `crates/veil-cli` をビルドします。GitHub CIのチェックアウトには未コミットのローカル変更は含まれません。
生成しただけでは、候補のビルド・動作を検証したことにはなりません。

別のリポジトリでVeilを使う場合は、レビュー済みで取得可能なVeilの完全なコミットSHAを指定します。
環境変数 `VEIL_COMMIT_SHA` に、レビュー済みの実際の40桁SHAを設定してから実行します。

```bash
veil init --ci github --ci-source "$VEIL_COMMIT_SHA"
```

SHAの存在・取得可否は生成時には問い合わせません。生成後に、そのリビジョンから取得するCLIのビルドと動作を確認してください。
`--pin-tag vX.Y.Z` も明示指定時だけ使えますが、タグは現在の作業ツリーとの同一性を証明しません。取得元の固定には完全なコミットSHAを使ってください。
`--pin-tag auto` / `none` と、ブランチ名・短いSHAは拒否します。取得元の2方式を同時指定することもできません。

## レポートと失敗条件

生成CIはインストール先・ビルド出力・HTMLレポートをrunnerの一時領域へ置き、スキャン対象はチェックアウトした作業ディレクトリとします。
レポートは `$RUNNER_TEMP/veil-report/veil-report.html`、artifactも同じファイルを参照します。
一時領域や実際のレポートディレクトリがスキャン対象内を指す場合は、準備処理で終了コード2として拒否します。
1回のスキャンでマスキングしたHTMLを生成し、High以上の検知を失敗条件にします。別途設定したスコア条件や上限も適用されます。
検知・打ち切りで失敗した場合も生成済みレポートのアップロードを試みる設定です。マスキングは完全な匿名化ではないため、利用前にartifactの閲覧範囲を確認してください。

一次資料: [GitHubの変数](https://docs.github.com/en/actions/reference/workflows-and-actions/variables)、[Cargo install](https://doc.rust-lang.org/cargo/commands/cargo-install.html)。
現在の公開・検証状態は [リリース条件](../ci/oss-cli-release.md)を参照してください。
