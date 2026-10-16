# 展開済みZIPからのローカル導入

Status: `reference`

[English](install.en.md)

この手順はM1開発ソースから新しく生成する候補ZIP向けです。公開済み0.21.0・rc.4のZIPには `install.sh` は入っていません。公開済みZIPは差し替えず、[従来の取得・直接実行](getting-started.md#ubuntuで自分のコードを試す)も引き続き使えます。0.22.0は未公開です。

## 導入する

対象はLinux x86_64・macOS ARM64です。配布前にLinux版のmusl静的リンクも検証します。macOSはad-hoc署名の試用版で、Developer ID署名・公証・ダウンロード後のGatekeeper動作は未検証です。OSの実行保護を無効化する手順は含めません。

Releaseの配布元・外側のSHA256SUMSを確認してから、新しいディレクトリへZIPを展開します。チェックサムの一致だけでは発行者の真正性を証明しません。必要に応じて信頼する公開鍵で署名タグを確認し、タグ内のZIPのSHA-256と照合してください。

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

候補の版番号は同梱CLIの実際の表示に従います。M1開発時点では0.21.0のままであり、0.22.0公開を意味しません。確認後は[初回スキャン→commit前点検→次回利用](getting-started.md#自分のファイルを一つスキャンする)へ進めます。CLIの `--lang` 対応は別のM2で、今回の `--lang` は導入スクリプト専用です。

## 衝突・失敗・以前の版への復帰

同じ版・同じ内容の再実行は成功します。異なる内容の同一版ディレクトリ、通常ファイルの既存CLI、管理外・破損リンクは上書きしません。別prefixを選ぶか、既存CLIをどう扱うか手元で判断してください。

このスクリプトの規約で導入した別版だけは、切替を明示できます。版の大小は自動判断せず、ダウングレードも明示操作です。

```sh
sh install.sh --lang ja --activate
```

以前の導入ZIPを保持しておけば、そのZIPの `install.sh --activate` と同じprefixで元の版へ戻せます。スクリプトのない旧0.21.0等は元の保存先から直接実行するか、以前のPATH設定へ手動で戻します。管理外CLIを `--activate` で上書きすることはできません。

失敗は終了コード2です。コピー完了・検証前に既存リンクを切り替えません。失敗した一時配置 `.install.*` や切替用リンクが残る場合は、表示した場所を確認してください。再試行時は新しい一時配置を使い、既存版の削除・自動アンインストール・自動掃除は行いません。切替失敗後は `command -v veil` と版表示で状態を確認してください。
