# veil scan

Status: `reference`

通常CLIの読取りスキャンです。初回利用は [チュートリアル](getting-started.md)、結果の扱いは [利用ガイドライン](usage-guidelines.md)。

```bash
veil scan [OPTIONS] [PATHS]...
```

| オプション | 振る舞い |
| --- | --- |
| `[PATHS]...` | 既定は `.`。存在するファイル・ディレクトリを指定。欠損パスはエラー |
| `--format <FORMAT>` | `text`（既定）・`json`・`html`・`table`・`md`・`markdown`。最小ビルドではtableはtextへフォールバック |
| `--fail-on-findings <N>` | **新規検知数がN以上**なら終了コード1。1件以上なら `1`。`0` は入力エラー（終了コード2） |
| `--fail-on-score <SCORE>` | 新規検知に指定スコア以上があれば1 |
| `--fail-on-severity <LEVEL>` | 新規検知に指定重大度以上があれば1 |
| `--baseline <PATH>` | baselineに一致する既存検知を抑制。抑制後の件数が0でも収集時の上限到達は保持 |
| `--write-baseline <PATH>` | 上限打ち切り・サイズ上限・読込みエラーなら2で保存せず、既存ファイルを保全。完全なスキャンなら保存（同名ファイルを上書き）。`--baseline` / `--interactive` と同時指定不可 |
| `--limit <N>` | 検知数上限。通常の非対話スキャンで上限到達は2。`0` は無制限。baseline作成も保存前に拒否 |
| `--mask-mode <MODE>` | `redact`・`partial`・`plain`。`plain` は生データを許可 |
| `--unsafe` | 生データ表示を許可。共有用に使わない |
| `--no-color` | 色を無効にする |
| `--staged` | Gitインデックスの対象を検査 |
| `--commit <SHA>` | 第1親との差分にある追加・変更ファイルを検査。最初のコミットは空ツリーとの差分で検査 |
| `--since <TIME>` | 指定時刻以降の各コミットの追加・変更ファイルを検査。最初のコミットも対象 |
| `--config <PATH>` | 設定ファイルを指定 |

全オプションは `veil scan --help` で確認します。組織・ユーザー・プロジェクト設定と環境変数も適用されます。
ファイル数上限は `core.max_file_count`、サイズ上限は `core.max_file_size`、検知上限は `output.max_findings` でも設定できます。

## 出力と失敗判定

設定がない既定のスキャンは、検知だけでは失敗しません。CIには失敗条件を明示します。

```bash
veil scan src --fail-on-findings 1
veil scan src --format json --fail-on-severity High
```

JSONは `veil-v1` の単一オブジェクトを標準出力へ返し、診断は標準エラーへ返します。実行エラー時はJSONがない場合があります。
終了コードは0（失敗条件非該当）、1（検知条件該当）、2（実行エラー・上限打ち切り）。
無視・バイナリによるスキップがあるため、0だけでは全件スキャンを保証しません。
baseline適用と上限を併用した通常の非対話スキャンでは、新規検知0件でも上限到達は2です。対話モードは不完全な結果をレビュー前に終了コード2で拒否します。Git blobやHEADツリー等を読めない検査はエラー2となり、baselineを保存しません。サブモジュールへのGitlinkは親リポジトリのblobではないため、明示的なスキップとして集計します。修正の [検証範囲](../ci/oss-cli-release.md#公開前の必須課題) を確認し、CIは非対話で実行してください。

レポートは対象の外へ保存します。既存出力を保全する場合は新しいファイル名を指定します。

```bash
veil scan src --format json > reports/scan.json
veil scan src --format html > reports/scan.html
```

`src` と既存の `reports` を自分のパスへ置き換えてください。baseline作成も上限到達時は終了コード2で拒否し、部分結果を保存せず既存ファイルを保全します。
通常スキャンのJSON契約は [こちら](../json-schema.md)。GitのHTTPS/SSHトランスポートは通常CLIに含まれず、履歴検査はローカルリポジトリが対象です。
