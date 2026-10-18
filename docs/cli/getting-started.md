# はじめてのローカルスキャン

[0.22.0正式版](https://github.com/mt4110/veil-rs/releases/tag/v0.22.0)は公開済みです。以下は正式版の取得手順です。別インデックスとフックの修正、氏名ルールの限定的な誤検出修正を含みます。

Status: `reference`

まず自分のソースファイルを一つ検査し、結果を読めるところまで進めます。その後、Gitのステージ済み変更を点検し、次の変更でも同じ手順を使います。設定生成・baseline・CI導入は初回には不要です。
公開済みZIPは作成時点の文書を同梱し、正式版準備中の記述やrc.4の取得手順が残ります。取得はこの最新版の手順を使用してください。本文書の更新で既存ZIPは差し替えません。

公開済み0.22.0のZIP、または未公開0.22.1候補のZIPを取得済みで `install.sh` がある場合は、[ローカル導入・次のターミナルのPATH設定](install.md)を選べます。以前のZIPは直接実行も可能ですが、表示される版番号は取得元Releaseと照合してください。下の取得手順は0.22.0用です。

## 0.22.1候補の導入から始める

[English tutorial](getting-started.en.md)。0.22.1は準備中で、まだ公開されていません。候補ZIPを取得済みの場合、外側のチェックサムと配布元を確認し、新しい場所へ展開してください。展開先で実行します。

```sh
sh install.sh --lang ja
```

[導入・PATH・衝突時の対処と復帰](install.md)に従い、表示されたPATH行を使用中のシェル設定へ一度だけ登録します。スクリプトは設定ファイルを編集しません。新しいターミナルで次を確認します。

```sh
command -v veil
veil --version
veil --lang ja scan --help
veil_bin=$(command -v veil)
```

意図した保存先と `veil 0.22.1` を確認し、失敗した場合は止めてください。Rust・Python・sudoは不要で、ステージ点検時だけGitが必要です。続いて下の「自分のファイルを一つスキャンする」→「commit前にステージ済み変更を点検する」へ進みます。日英切替はhelpを対象にし、実行結果全般の翻訳ではありません。macOSのad-hoc署名制限は正式版0.21.0と同様です。

以下の取得手順は公開済み0.22.0用です。0.22.1候補を導入済みなら再取得は不要です。

## 正式版ZIPを取得済みの場合

Release本文の配布元コミットとZIP全体のSHA256SUMS、同梱のライセンス表記を確認して、新しいディレクトリへ展開します。既存CLIやプロジェクトを上書きしません。

展開したディレクトリで次を実行し、`veil 0.22.0` を確認してください。続いて「自分のファイルを一つスキャンする」から進みます。未取得なら下の取得手順を使用します。

```bash
veil_bin="$PWD/veil"
"$veil_bin" --version
```

macOS ARM64版はad-hoc署名の試用版です。Developer ID署名・公証・ダウンロード後のGatekeeper動作は未検証です。旧rc.4のバイナリは版番号が異なり、正式版と読み替えません。[旧rc.4](https://github.com/mt4110/veil-rs/releases/tag/v0.21.0-rc.4)は公開済みプレリリースとして保持しています。

## Ubuntuで自分のコードを試す

### 取得してチェックサムを確認する

対象はUbuntuのx86_64環境です。0.22.0のLinux版はx86_64実機とGitHub-hosted CIで検証しています。他のCPU・ディストリビューション・旧カーネルの動作はこの検証から保証しません。
`curl`・`unzip`・`sha256sum` が必要です。取得時はネット接続が必要ですが、組み込みルールによる通常のスキャンは外部APIを呼びません。RustやPythonのインストールはこの初回手順には不要です。

以下は新しい保存先を作り、既存CLIやプロジェクトを上書きせず展開します。途中でエラーになったら次へ進まず、取得・チェックサム確認・展開のどこで止まったかを確認してください。

```bash
trial_dir=$(mktemp -d "$HOME/veil-0.22.0.XXXXXX") &&
cd "$trial_dir" &&
release_url="https://github.com/mt4110/veil-rs/releases/download/v0.22.0" &&
zip_name="veil-0.21.0-x86_64-unknown-linux-musl.zip" &&
curl -fLO "$release_url/$zip_name" &&
curl -fLO "$release_url/SHA256SUMS" &&
sha256sum --ignore-missing -c SHA256SUMS &&
unzip "$zip_name" -d cli &&
veil_bin="$PWD/cli/veil" &&
"$veil_bin" --version &&
printf 'CLIの保存先: %s\n' "$veil_bin"
```

ZIP名に対する `OK` とバージョン `0.22.0` を確認してください。外側の `SHA256SUMS` にはmacOS ZIPも載るので、Linux ZIPのみ取得する手順では `--ignore-missing` を使います。チェックサムは配布された値との一致を確認するもので、独立した発行者署名の検証ではありません。Linux ZIP用の署名ファイルは添付していませんが、署名タグ `v0.22.0` のメッセージに両ZIPのSHA-256を含みます。真正性を確認する場合は、信頼する公開鍵でタグ署名を検証し、その値と照合します。
表示したCLIの絶対パスを控えてください。ZIP内のREADME・docs・ライセンス文書はオフラインでも読めます。macOS ARM64版はad-hoc署名の試用版で、Developer ID署名・公証・ダウンロード後のGatekeeper動作は未検証です。Windows版は0.22.0の配布対象外です。

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

### commit前にステージ済み変更を点検する

最初のファイルの結果を理解したら、Gitのプロジェクト内で次の点検を行います。Gitが必要です。既存のインデックスや設定は自動で変更しません。
まず、commitしたいファイルだけを自分でステージします。下の `main.py` は実際のファイル名へ置き換えてください。`git add .` で不要なファイルまでまとめて追加しないでください。

```bash
git status --short
git add -- main.py
git diff --cached --name-only
```

Gitのコマンドにエラーがあれば、ここで止めてください。表示されたファイル一覧が意図した対象であることを確認してから実行します。

```bash
"$veil_bin" scan --staged --mask-mode redact --no-color --fail-on-findings 1
scan_status=$?
printf '終了コード: %s\n' "$scan_status"
```

`--staged` はHEADとの差分にある追加・変更ファイルの**インデックス内の内容全体**を読みます。追加行だけの検査ではなく、作業中のファイルの内容とも異なります。既にステージされている他のファイルも対象です。ファイルパスによる絞り込みを前提にせず、Gitプロジェクトのルートで実行してください。

- `0`: 検出条件に該当しない。空のステージでも0になるので、対象一覧と検査済み件数を確認する。
- `1`: 検出条件に該当。内容を手元で確認し、必要な修正後に同じファイルを `git add -- main.py` で**再ステージ**してから点検する。
- `2`: 実行エラー・不完全。Git外での実行や読取り異常などを解消し、再実行する。

作業ファイルだけを修正してもインデックスには反映されません。例えば作業ファイルの点検が0でも、再ステージ前の `--staged` は古い内容を検出して1になることがあります。逆に、未ステージの変更はこの点検では検査されません。

| 対象 | この点検の範囲 |
| --- | --- |
| 追加・変更され、ステージされたテキスト | インデックス内のファイル全体を検査 |
| 未ステージの変更・未追跡ファイル・HEADから変更のないファイル | 対象外。必要なら通常のファイルスキャンで別に確認 |
| `.gitignore` の対象 | 対象外になる。このCLIでは `git add -f` で追加しても点検を保証しない |
| バイナリ・サブモジュール参照 | スキップ。Git履歴全体やサブモジュール内部もこの手順では検査しない |

疑わしいテキストが対象外の場合は、通常のファイルスキャンで明示して確認できます。既存ignoreやbaselineを消して点検を通す手順にはしません。
**0はcommit・pushの安全保証ではありません。** この手順はGit commit/pushを自動実行しません。

公開済みrc.2と旧rc.3候補（配布元04745e7）は `GIT_INDEX_FILE` を反映しません。別インデックスや部分コミット時の自動フックを、通常の `git add` 後の手動点検と同一視しないでください。現行ソースでは修正しており、仕様は [scanのインデックス選択](scan.md#gitが選ぶインデックス現行ソースの修正) と [フック導入](README.md#コミット前フック現行ソースの修正) を参照してください。

### 次のコード変更でも同じ点検を使う

次にコードやAPI設定を変更したとき、commit前にこの点検を繰り返します。0.22.1候補を導入してPATHを設定した場合、新しいシェルで `command -v veil` と `veil --version` を確認し、`veil_bin=$(command -v veil)` で同じ版を選びます。直接実行の場合は、控えたCLIの保存先を設定し直し、プロジェクトへ移動します。下の両方のパスは実際のものに置き換えてください。

```bash
veil_bin="/absolute/path/to/veil-0.22.0.XXXXXX/cli/veil"
cd "/absolute/path/to/your-project"
```

移動に成功したことを確認し、上の「commit前にステージ済み変更を点検する」と同じ順序で、対象選択 → `git add` → 対象一覧確認 → `scan --staged` を行います。検出を修正した場合は再ステージしてもう一度点検します。
commit前の新規ファイル単体の確認には、初回と同じ通常スキャンも使えます。CIや自動フックの導入は、この手動の結果を理解してから検討できます。次の変更がなければ、再利用のためだけに作業を作る必要はありません。

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

現在の配布にはWindows ZIPはありません。以下の配布実行例はLinux x86_64とmacOS ARM64向けです。
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
公開リポジトリからの取得例は [README](../../README.md)。0.22.0の配布版は公開済みです。ソースの後続変更と公開タグ・配布ZIPの内容は区別してください。

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
