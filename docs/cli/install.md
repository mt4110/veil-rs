# 展開済みZIPからのローカル導入

Status: `reference`

[English](install.en.md)

[v0.22.2](https://github.com/mt4110/veil-rs/releases/tag/v0.22.2)、v0.22.1、v0.22.0 の ZIP には導入スクリプトが入っています。v0.21.0 と rc.4 は展開した CLI を直接実行し、取得元 Release の版番号と照合してください。

## ダウンロードを検証する

以下を新しい空のディレクトリで実行します。curl、SSH 署名に対応した OpenSSH、GitHub CLI（認証が求められたらログイン）、sha256sum または shasum が必要です。既定は Linux x86_64 musl 版です。macOS arm64 版は最初の ZIP 指定をコメントにある値へ置き換えてください。チェックサムは取得した ZIP と SHA256SUMS の一致を、分離署名は配布用鍵による ZIP の署名を、SLSA 来歴証明は同じ ZIP と指定の GitHub Actions ワークフロー・配布元コミットの結び付きを確認します。期待する鍵のフィンガープリントは、信頼するチェックアウトや経路で得た[配布用鍵のフィンガープリントの記録](https://github.com/mt4110/veil-rs/blob/7dbb3fa249c32315c33209b1ad87b00018857d5f/docs/ci/oss-cli-release.md#検証する)と照合してください。

```sh
(
  set -eu
  zip=veil-0.22.2-x86_64-unknown-linux-musl.zip
  # macOS arm64: zip=veil-0.22.2-aarch64-apple-darwin.zip
  repo=mt4110/veil-rs
  source_sha=7dbb3fa249c32315c33209b1ad87b00018857d5f
  release_url="https://github.com/$repo/releases/download/v0.22.2"
  for asset in "$zip" "$zip.sig" "$zip.intoto.jsonl" SHA256SUMS; do
    curl -fLO "$release_url/$asset"
  done

  awk -v zip="$zip" '$2 == zip {print; n++} END {if (n != 1) exit 1}' \
    SHA256SUMS > selected.sha256
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum -c selected.sha256
  else
    shasum -a 256 -c selected.sha256
  fi

  key=veil-rs-release-ed25519.pub
  curl -fLO "https://raw.githubusercontent.com/$repo/$source_sha/docs/security/$key"
  fingerprint=$(ssh-keygen -lf "$key" -E sha256 | awk 'NR == 1 {print $2}')
  test "$fingerprint" = 'SHA256:UdvtwJ2HTocMm2555crXrgGdHEd30wtRO7LZT65X7nE'
  awk 'NR == 1 {print "veil-rs-release namespaces=\"veil-rs-release\" " $1 " " $2}' \
    "$key" > allowed_signers
  ssh-keygen -Y verify -f allowed_signers -I veil-rs-release \
    -n veil-rs-release -s "$zip.sig" < "$zip"

  gh attestation verify "$zip" --bundle "$zip.intoto.jsonl" --repo "$repo" \
    --signer-workflow "$repo/.github/workflows/release_provenance.yml" \
    --source-ref refs/heads/main --source-digest "$source_sha" \
    --predicate-type https://slsa.dev/provenance/v1
)
```

どれかが失敗した場合は ZIP を展開・実行せずに止めてください。すべて通ったら新しいディレクトリへ展開し、同梱のライセンス表記を保管します。

## 導入する

macOS の実行ファイルは ad-hoc 署名です。ZIP の分離署名は Developer ID 署名や公証に代わるものではなく、ダウンロード後の Gatekeeper の動作は未検証です。OS の実行保護は有効のままにしてください。

展開したディレクトリで実行します。

```sh
sh install.sh --lang ja
```

`sh`、`awk`、`sha256sum`または`shasum`、`cp`・`mv`・`ln`等のOS標準ツールを使います。Rust・sudo・Python・ネット接続は不要です。スクリプトは内部SHA256SUMSを検証し、OS/CPUとCLI版表示を確認します。シンボリックリンクや不正な内部パスは拒否し、コピー後にも照合します。

既定の配置先は `~/.local/share/veil/<version>/<platform>/`、呼出先は `~/.local/bin/veil` です。platformは `x86_64-linux` または `aarch64-darwin` という導入用の名前で、RustのABI target tripleではありません。CLIと文書、ライセンス原文、第三者表記・必要なソース、チェックサムを一式保持します。

別の配置先を使う場合は、絶対パスのprefixを指定します。

```sh
sh install.sh --prefix "$HOME/veil-tools" --lang ja
```

prefixにはシンボリックリンク・`.`/`..`・改行・PATH区切りのコロンを含めません。空白や単一引用符を含むパスは扱えます。相対パス・ルート `/` は拒否します。

## PATHを一度登録する

スクリプトは登録行を表示しますが、`.zshrc`・`.zprofile`・`.bashrc`・`.bash_profile`・`.profile` を編集しません。表示したprefixの登録行を使ってください。既定prefixなら次の行です。

```sh
export PATH="$HOME/.local/bin:$PATH"
```

- zshの対話ターミナル: 利用中の `~/.zshrc` に一度登録します。
- bashの対話・非ログインターミナル: `~/.bashrc` に登録します。
- bashのログインターミナル: 読み込んでいる `~/.bash_profile`・`~/.bash_login`・`~/.profile` のいずれかへ登録します。既存profileが `.bashrc` を読んでいれば同じ行を重ねて登録する必要はありません。

既にbinディレクトリがPATHに含まれる場合は追記不要です。新しいターミナルを開いて、呼ばれた実体と版を確認します。期待した保存先でなければ、同名CLIやPATHの順序を確認してください。

```sh
command -v veil
veil --version
```

同梱CLIの表示が取得元Releaseと一致することを確認します。最新の正式版は `veil 0.22.2` と表示します。確認後は[初回スキャン→commit前点検→次回利用](getting-started.md#自分のファイルを一つスキャンする)へ進めます。導入スクリプトとCLIの言語は別々に指定します。CLI helpの言語設定は下記を参照してください。

## 衝突・失敗・以前の版への復帰

同じ版・同じ内容の再実行は成功します。異なる内容の同一版ディレクトリ、通常ファイルの既存CLI、管理外・破損リンクは上書きしません。別prefixを選ぶか、既存CLIをどう扱うか手元で判断してください。

このスクリプトの規約で導入した別版だけは、切替を明示できます。版の大小は自動判断せず、ダウングレードも明示操作です。

```sh
sh install.sh --lang ja --activate
```

以前の導入ZIPを保持しておけば、そのZIPの `install.sh --activate` と同じprefixで元の版へ戻せます。スクリプトのない旧0.21.0等は元の保存先から直接実行するか、以前のPATH設定へ手動で戻します。管理外CLIを `--activate` で上書きすることはできません。

失敗は終了コード2です。コピー完了・検証前に既存リンクを切り替えません。失敗した一時配置 `.install.*` や切替用リンクが残る場合は、表示した場所を確認してください。再試行時は新しい一時配置を使い、既存版の削除・自動アンインストール・自動掃除は行いません。切替失敗後は `command -v veil` と版表示で状態を確認してください。

## CLI helpの言語

公開済み0.22.2・0.22.1・0.22.0では `veil --lang ja --help` と `veil --lang en help scan` でhelpを切り替えられます。公開済み0.21.0 / rc.4にはこのCLIオプションはありません。導入スクリプトとCLIの言語指定は別々です。

優先順は `--lang` > `VEIL_LANG` > `LC_ALL` > `LC_MESSAGES` > `LANG` > 英語です。空の環境値は無視し、localeは最初の非空値だけを採用します。`ja`系localeは日本語、それ以外は英語です。明示指定・`VEIL_LANG`の不正値は終了2になります。有効な明示指定は不正な環境値に優先します。

公開19画面の `-h` は要約、`--help` と `help <command>` は例と注意を表示します。引数なしでも同じ言語のhelpを表示します。検査結果・JSON/SARIF・実行エラー全般の翻訳ではありません。`init --wizard` の言語選択は従来どおり別機能です。[英語チュートリアル](getting-started.en.md)も同梱します。合成入力の受入確認と実作業での継続利用実績は区別します。
