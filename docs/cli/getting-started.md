# はじめてのローカルスキャン

Status: `reference`

まず自分のソースファイルを一つ検査し、結果を読めるところまで進めます。設定生成・baseline・CI導入は初回には不要です。
0.21.0-rc.1は [公開済みプレリリース](https://github.com/mt4110/veil-rs/releases/tag/v0.21.0-rc.1) です。この文書の後続修正は、既存の配布ZIP内の文書には自動反映されません。

## Ubuntuで自分のコードを試す

### 取得してチェックサムを確認する

対象はUbuntu 24.04のx86_64環境です。RCはUbuntu 24.04.5 LTSの実機で検証しています。他のCPU・ディストリビューション・旧カーネルの動作はこの検証から保証しません。
`curl`・`unzip`・`sha256sum` が必要です。取得時はネット接続が必要ですが、組み込みルールによる通常のスキャンは外部APIを呼びません。RustやPythonのインストールはこの初回手順には不要です。

以下は新しい保存先を作り、既存CLIやプロジェクトを上書きせず展開します。途中でエラーになったら次へ進まず、取得・チェックサム確認・展開のどこで止まったかを確認してください。

```bash
trial_dir=$(mktemp -d "$HOME/veil-rc1.XXXXXX") &&
cd "$trial_dir" &&
release_url="https://github.com/mt4110/veil-rs/releases/download/v0.21.0-rc.1" &&
zip_name="veil-0.21.0-rc.1-x86_64-unknown-linux-musl.zip" &&
curl -fLO "$release_url/$zip_name" &&
curl -fLO "$release_url/SHA256SUMS" &&
sha256sum --ignore-missing -c SHA256SUMS &&
unzip "$zip_name" -d cli &&
veil_bin="$PWD/cli/veil" &&
"$veil_bin" --version &&
printf 'CLIの保存先: %s\n' "$veil_bin"
```

ZIP名に対する `OK` とバージョン `0.21.0-rc.1` を確認してください。外側の `SHA256SUMS` にはmacOS ZIPも載るので、Linux ZIPのみ取得する手順では `--ignore-missing` を使います。チェックサムは配布された値との一致を確認するもので、独立した発行者署名の検証ではありません。
表示したCLIの絶対パスを控えてください。ZIP内のREADME・docs・ライセンス文書はオフラインでも読めます。macOS ARM64版はad-hoc署名の試用版で、Developer ID署名・公証・ダウンロード後のGatekeeper動作は未検証です。Windows版はこのRCの配布対象外です。

### 自分のファイルを一つスキャンする

自分が書いた、存在するソースファイルを一つ選びます。下のプロジェクトの絶対パスと `main.py` を実際のものへ置き換えてください。プロジェクトへ移動することで、その場所の既存設定を使います。

```bash
cd "/absolute/path/to/your-project" &&
"$veil_bin" scan "main.py" --mask-mode redact --no-color --fail-on-findings 1
scan_status=$?
printf '終了コード: %s\n' "$scan_status"
```

`cd` が失敗した場合はスキャンしていません。移動に成功した場合は、次のように読みます。

| 終了コード | 意味 | 次の行動 |
| --- | --- | --- |
| `0` | 指定した失敗条件に該当しない | 対象・スキップ状況を確認し、必要なら対象を自分のソースディレクトリへ広げる |
| `1` | 検出条件に該当し、スキャンは完了 | 手元でルール・位置を確認する。本物の秘密なら失効・再発行・コード修正を判断する |
| `2` | 実行エラーまたは不完全なスキャン | 診断を読み、設定・入力・上限などの問題を解消して再実行する |

初回完了の条件は、自分のファイルを検査し、結果を理解し、次の行動を決められたことです。終了コード1をツールの起動失敗と混同しないでください。0や検出ゼロは、秘密情報がないことの保証ではありません。
`scan` はソースを書き換えません。この手順は現在の指定ファイルを対象とし、Git履歴全体は検査しません。無視設定や隠しファイル等により対象外になる入力があります。`scan .` だけで `.env` を含めた全ファイルを点検したとは扱わないでください。

組織・ユーザー設定や `VEIL_*` 環境変数は結果に影響します。既存設定やbaselineを削除・作り直さず、必要なら `"$veil_bin" config check` で確認してください。詳細は [利用ガイドライン](usage-guidelines.md)。
`--mask-mode redact` でもパスや未検出の文字列は残り得ます。コード、実際の秘密値、スキャン結果全文を公開・送信する必要はありません。

### 次のコード変更でも使う

新しいシェルでは、控えた保存先を使って変数を設定し直します。下のCLIパスも実際の保存先へ置き換えてください。

```bash
veil_bin="/absolute/path/to/veil-rc1.XXXXXX/cli/veil"
cd "/absolute/path/to/your-project" &&
"$veil_bin" scan "main.py" --mask-mode redact --no-color --fail-on-findings 1
scan_status=$?
printf '終了コード: %s\n' "$scan_status"
```

次にコードやAPI設定を変更したとき、push前に再実行します。ソースディレクトリへ対象を広げる場合も、結果とスキップ状況を確認してください。Gitのステージ済みファイルを検査する `--staged` やCI導入は、手動の結果を理解してから検討できます。

## 任意: 架空入力で機能を学ぶ

以下は新しい作業ディレクトリで、検知・マスキング・baseline・CI終了条件を学ぶための手順です。初回完了のために実行する必要はありません。
実際の認証情報は使いません。macOS/Linuxのシェルと、サンプル生成用のPython 3を使います。Windowsでの実行は未検証です。
上で取得したCLIを使う場合は、同じシェルで `export PATH="$(dirname "$veil_bin"):$PATH"` を実行すると、以下の `veil` コマンドで呼び出せます。

## 1. CLIを用意する

### 配布アーカイブの場合

展開したディレクトリで、macOS/Linuxなら以下を実行します。Rustによる再ビルドは不要です。
このシェルのPATHへ追加することで、次の手順で作業ディレクトリを移動しても同梱CLIを呼び出せます。

```bash
export PATH="$PWD:$PATH"
veil --version
veil scan --help
```

Windowsでは同梱 `veil.exe` の絶対パスを使うか、展開先をシェルのPATHへ追加します。
以降のPythonによる架空入力作成にはPython 3が必要です。ソースからビルドする場合だけ、次の手順を使います。

### ソースからインストールする場合

Rust 1.92.0以上と、Rustおよび同梱Cライブラリのビルドに必要なOSの開発ツールを用意します。
取得したソースのルートで実行します。

```bash
cargo install --locked --path crates/veil-cli
veil --version
veil scan --help
```

`cargo install` のインストール先がPATHに入っている必要があります。既存のveilがある場合は、まず `veil --version` と実行ファイルの場所を確認してください。
既存インストールを変更せず試す場合は、新しいディレクトリを `--root` に指定し、そこに生成された `bin/veil` を使えます。
初回の依存取得にはネットワークが必要です。スキャン処理自体は外部APIを呼びません。
公開リポジトリからの取得例は [README](../../README.md)。0.21.0-rc.1の配布版は公開済みです。ソースの後続変更と公開タグ・配布ZIPの内容は区別してください。

組織・ユーザー設定や `VEIL_*` 環境変数がある場合は結果に影響します。既存設定を削除せず、`veil config check` と `veil config dump` で確認します。
設定の表示結果にはローカルのパス等が含まれるため、そのまま公開しないでください。

## 2. テスト専用の入力を作る

既存のディレクトリと重ならない名前で作成します。以下の名前が存在する場合は別名にしてください。
レポートとbaselineをスキャン対象の `input` の外へ置き、出力を次のスキャンへ混入させません。

```bash
mkdir veil-tutorial
cd veil-tutorial
mkdir input reports
python3 - <<'PYTHON'
from pathlib import Path
sample = "AKIA" + "A" * 16
Path("input/sample.txt").write_text("aws_key=" + sample + "\n", encoding="utf-8")
PYTHON
```

これは検出パターンに合わせた架空の値です。実際のキーやパスワードを貼り付けないでください。

## 3. 人間向け出力とJSONを読む

```bash
veil scan input --no-color
veil scan input --format json > reports/scan.json
```

設定・環境変数で失敗条件を指定していない場合、検知があってもこのスキャンは終了コード0です。
テキストにはルールID・位置・重大度・スコア・マスキングした行が表示されます。同じ行が複数のルールで検知される場合があります。
JSONでは `schemaVersion` が `veil-v1`、`summary.new_findings` が1以上で、`findings` に `creds.aws.access_key_id` が含まれます。件数や所要時間の固定値は契約にしません。
既定では検知した値をマスキングしますが、パスや検知されなかった文字列まで匿名化する機能ではありません。

`summary.scanned_files` と `skipped_files`、`limit_reached` と `file_limit_reached` を確認してください。
無視設定・バイナリ・サイズ上限でスキップするファイルがあります。詳細は [利用ガイドライン](usage-guidelines.md) と [JSON契約](../json-schema.md)。

## 4. 1件以上の新規検知で失敗させる

```bash
veil scan input --fail-on-findings 1
```

この入力では終了コード1を期待します。件数条件は「指定値以上」で、`0` は無効な入力として終了コード2になります。
実行エラー・通常スキャンの上限打ち切りは終了コード2です。終了コード1を実行エラーと混同しないでください。

## 5. 既存検知をbaselineに記録する

まず通常スキャンで両方の上限フラグがfalseであることを確認し、入力を変更せずに実行します。
保存先は新しいファイルにしてください。`--write-baseline` は既存の同名ファイルを上書きします。

```bash
veil scan input --write-baseline reports/veil.baseline.json
veil scan input --baseline reports/veil.baseline.json --fail-on-findings 1
```

最初のコマンドはbaselineを保存し、標準エラーに件数を表示します。通常のスキャンレポートは出力しません。
2つ目は同じ検知を抑制し、終了コード0になります。相対パスで照合するため、同じ作業ディレクトリと対象パスで実行します。

通常の非対話ファイルスキャンで検知数・ファイル数・サイズ上限、読込み/UTF-8エラーがある場合、作成コマンドは終了コード2で拒否し、部分結果を保存しません。新規ファイルを作らず、既存baselineも変更しません。無視・バイナリによるスキップは別に確認するため、終了コード0だけで全ファイルの確認を保証するものではありません。
baselineは秘密そのものを識別する方式でもありません。[照合方式と制約](../baseline/usage.md)を確認し、検知済みの本物の秘密は失効・修正してください。

## 6. 新しいファイルの検知を確認する

```bash
python3 - <<'PYTHON'
from pathlib import Path
sample = "AKIA" + "C" * 16
Path("input/new.txt").write_text("aws_key=" + sample + "\n", encoding="utf-8")
PYTHON
veil scan input --baseline reports/veil.baseline.json --fail-on-findings 1
```

既存検知は抑制され、新しいファイルの検知によって終了コード1になります。
`--limit` や設定の `output.max_findings` に達した通常の非対話スキャンは、baseline適用後に新規件数が0でも終了コード2になります。CIはこの2を成功に変換しません。対話モードの不完全判定とGit読取り失敗時の保存拒否は追加修正済みで、Linux x86_64 musl候補でも終了コード2とbaseline保全を確認しています。[修正後の検証範囲](../ci/oss-cli-release.md#公開前の必須課題) も確認してください。
ここでbaselineを作り直すと新しい検知も抑制されるため、CIを通す目的だけで作り直さないでください。

## 7. 設定は必要になったら作る

```bash
veil init --non-interactive
veil config check
veil doctor
veil rules list
veil rules explain creds.aws.access_key_id
```

初期化は任意です。Application設定にはスコアによる失敗条件等が入り、設定前と終了コードが変わる場合があります。
`--non-interactive` は既存 `veil.toml` を保全してエラーにします。生成内容を確認してから使ってください。[init](init.md)にプロファイルの差を記載しています。

## CIへの接続

CIのインストール段階で、検証済みの同じCLIとルールを用意してから、以下のスキャンを実行します。
このチュートリアルでは新規検知を追加した後なので、終了コード1を期待します。

```bash
veil scan input --baseline reports/veil.baseline.json --format json --fail-on-findings 1 > reports/ci.json
```

実プロジェクトでは `input` とbaselineの場所を、存在する対象パスへ置き換えます。
CIは0を成功、1を検知条件による失敗、2を実行エラー・不完全として扱い、エラーを成功へ変換しないでください。
CLIの取得元をレビュー済みのリビジョンへ固定し、公開前のローカル改修が過去の公開タグに含まれると仮定しないでください。
CI生成は取得元を必須にします。Veilソースを含むリポジトリでは `veil init --ci github --ci-source checkout`、別のリポジトリではレビュー済みの完全なVeilコミットSHAを `--ci-source` へ指定します。公開タグをmanifestから推測しません。生成CIのレポートはrunnerの一時領域へ保存します。生成しただけでは未コミットの候補を検証したことにはなりません。詳細は [init](init.md)と [リリース条件](../ci/oss-cli-release.md)。
