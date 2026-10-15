# Veil — ローカルで検出・マスキングするシークレットスキャナー

[0.21.0-rc.4](https://github.com/mt4110/veil-rs/releases/tag/v0.21.0-rc.4) は公開済みプレリリースです。既存の出力秘匿・安全な書換えに加え、別インデックスとフックの修正、氏名ルールの限定的な誤検出修正を含みます。以下の取得手順はrc.4を使用します。

Veil は、ソースコードやログに含まれるシークレットをローカルで検出・マスキングする Rust の OSS CLI です。
通常ビルドは外部API、ライセンス認証、テレメトリ送信を含みません。組み込みルールで、設定ファイルを作らずにスキャンを始められます。
検出結果には誤検知・見逃しがあり、秘密情報が存在しないことを保証するものではありません。

[English](README_EN.md) · [チュートリアル](docs/cli/getting-started.md) · [利用ガイドライン](docs/cli/usage-guidelines.md) · [CLI](docs/cli/README.md) · [開発ガイド](docs/DEVELOPMENT.md) · [アーキテクチャ](docs/architecture/local-oss.md)

## Ubuntuで自分のコードを初回スキャンする

GitHubに公開する前に、自分のPythonプロジェクトなどのソースファイルを一つ点検してみてください。Python以外のソースも対象にできます。
[0.21.0-rc.4](https://github.com/mt4110/veil-rs/releases/tag/v0.21.0-rc.4) のLinux x86_64 musl版を、x86_64実機とGitHub-hosted CIで検証しています。Rustのインストール、設定ファイルの生成、RulePack更新は初回には不要です。

Ubuntuのx86_64環境と `curl`・`unzip`・`sha256sum` を用意し、[チュートリアルの取得・チェックサム確認](docs/cli/getting-started.md#ubuntuで自分のコードを試す)から始めてください。取得したCLIの絶対パスを `veil_bin` に入れたら、自分のプロジェクトへ移動して実行します。

```bash
# 実際のプロジェクトの絶対パスと、存在するソースファイルへ置き換える
cd "/absolute/path/to/your-project" &&
"$veil_bin" scan "main.py" --mask-mode redact --no-color --fail-on-findings 1
scan_status=$?
printf '終了コード: %s\n' "$scan_status"
```

`cd` に失敗した場合はスキャン未実行です。成功した場合、終了コード `0` は検出による失敗条件に該当しない、`1` は検出がありスキャンは完了、`2` は実行エラー・不完全なスキャンです。`1` をインストール失敗と扱わず、手元で検出内容を確認してください。
初回の完了は、自分のファイルを検査し、結果を読んで次の行動を判断できた状態です。検出ゼロや終了コード0は安全性の保証ではありません。

`scan` はソースを書き換えません。指定した現在のファイルが対象で、Git履歴全体や無視されたファイルまで検査したとは扱いません。マスク後もパスや未検出の文字列は残り得るため、出力全文を公開しないでください。
初回の結果を理解したら、Gitのプロジェクト内でcommitしたいファイルだけをステージし、対象一覧を確認して点検します。

```bash
git add -- main.py
git diff --cached --name-only
# Gitの操作と対象一覧を確認してから実行する
"$veil_bin" scan --staged --mask-mode redact --no-color --fail-on-findings 1
scan_status=$?
printf '終了コード: %s\n' "$scan_status"
```

Git操作が失敗した場合は点検へ進まないでください。`--staged` は追加・変更ファイルのインデックス内の内容全体を読みます。修正後は再ステージが必要です。未ステージ・未追跡・変更のないファイルや無視対象まで確認したとは扱いません。空のステージでも終了コード0になります。
次の変更でもcommit前に同じ順序で点検します。CLIの保存先、対象外ファイル、結果の読み方は [初回から次回までのチュートリアル](docs/cli/getting-started.md#commit前にステージ済み変更を点検する) を参照してください。

今回のRCの配布対象はLinux x86_64 musl・macOS ARM64です。macOSはad-hoc署名で、Developer ID署名・公証・ダウンロード後のGatekeeper動作は未検証です。Windows・Intel Mac・Linux ARM64・LSP単体配布は対象外です。

## インストール

### 配布アーカイブを使う場合

展開したディレクトリには `veil`（Windowsは `veil.exe`）、このREADME、`docs/`、ライセンス文書、`SHA256SUMS` が入ります。
macOS/Linuxでは、そのディレクトリで以下を実行します。PATHの変更は現在のシェルだけに適用されます。

```bash
./veil --version
export PATH="$PWD:$PATH"
veil --help
```

Windows PowerShellでは `./veil.exe --version` で確認し、以降も実行ファイルの絶対パスを使うか、そのディレクトリをPATHへ追加します。
同梱の [チュートリアル](docs/cli/getting-started.md) と [利用ガイドライン](docs/cli/usage-guidelines.md) はネット接続なしで読めます。
ソースコードと開発用スクリプトは同梱しません。開発ガイドのビルド・検証はソースを取得してから行います。
0.21.0-rc.4のLinux x86_64 musl版・macOS ARM64版は公開済みです。公開は全OSの検証完了を意味しません。同梱文書はRC作成時の版で、このREADMEやチュートリアルの後続修正は既存ZIPへ自動では反映されません。

### Linuxの対応環境・最低条件

0.21.0-rc.4のLinux版はx86_64 musl静的リンクです。実機とGitHub-hosted CIで起動・配布検査・回帰を確認しています。
以下は0.21.0-rc.1当時の検証記録です。0.21.0-rc.1のx86_64候補は、Ubuntu 24.04.5 LTS（Linux 7.0.0-38-generic）のx86_64実機でネイティブビルド・起動・スキャン・マスキング・終了コード0/1/2を確認しました。
ELFのPT_INTERP・DT_NEEDED・GLIBC参照がなく、利用時のRust・共有libc・libz・libgccの追加インストールは不要です。ライセンス原文・ソース・チェックサム・同梱リンクも検査しました。
対話/Git読取り異常の修正後のmusl候補で、通常357件・最小356件・JP358件・LSP25件の全テストと、release構成の関連回帰84件が成功しました。配布ZIPから取り出したCLIの21ケースで、不完全時の終了コード2・部分結果非出力・新規baseline非作成・既存baseline保全を確認しています。CLI経由・単体のLSP stdio起動も確認しました。構成間のテストは重複するため合計を固有件数としません。
旧ARM64候補とQEMUによるx86_64候補の成功は、現行RCのARM64・他ディストリビューションの成功を示しません。
最低カーネル版は未確定で、旧カーネル・他ディストリビューション・別の実機/VM・他CPU・性能は未検証です。

前回のDebian 13製GNU候補はglibc 2.39以上とlibz.so.1 / libgcc_s.so.1が必要です。Ubuntu 22.04のglibc 2.35では起動に失敗しました。
CPU/配布方式を区別して候補を選んでください。作成・検査手順と詳しい制限は [Linux配布ガイド](docs/cli/linux-distribution.md) にまとめています。

### Windowsの対応環境・検証範囲

x86_64 Windows GNU候補をRust 1.92.0 / MinGW-w64で実ビルドし、Wine 10.0＋QEMU（ARM64実ホスト）で起動・マスキングJSON・終了コード0/1/2を確認しました。対象はWindows 10以降 / Server 2016以降ですが、Windowsネイティブと既存CIのMSVC候補は未検証です。
PEの通常/遅延importはWindows提供の12 DLLのみで、追加MinGW DLLは検出しませんでした。静的MinGW/GCCランタイムの原文も同梱します。実WindowsでのDLL解決・クリーン環境・署名は未検証です。[Windows配布ガイド](docs/cli/windows-distribution.md)に具体的なDLL名と制限を記載しています。
続く環境確認では、アクセス可能なWindows実機/VMを特定できず、ローカルのWindows SDK・MSVC CRTヘッダー・MSVC用リンカーも利用できませんでした。今回のMSVC候補のビルド・起動・DLL検査は未実施です。同ガイドにWindows環境での検証手順を記載しています。

### ソースからインストールする場合

以下はこのチェックアウトのソースに対する手順です。公開済みRCを試す場合は上の配布版手順を使ってください。mainの後続変更を公開タグに含まれるものとして扱わないでください。[リリース前の課題](docs/ci/oss-cli-release.md)を参照してください。

ソースからのインストールには Rust 1.92.0 以上が必要です。macOS の開発環境は `nix develop` を利用できます。
ネイティブ依存のビルドにCツールチェーンも必要です。通常の `cargo install` はその環境向けのビルドであり、Linuxの静的配布候補には専用のmusl targetと検査を使います。

```bash
git clone https://github.com/mt4110/veil-rs.git
cd veil-rs
cargo install --locked --path crates/veil-cli
veil --help
```

## まずスキャンする

```bash
veil scan .
veil scan . --format json
# CI: 新規検出が1件以上なら終了コード1
veil scan . --fail-on-findings 1
```

`scan` は通常、ファイルを読み取り、結果を標準出力へ表示します。JSON は `schemaVersion: "veil-v1"` を持ち、診断は標準エラーへ出力します。
結果は既定でマスキングします。CLIの `--unsafe`、`--mask-mode plain` は生の秘密情報を表示するため、保存・共有する出力には使わないでください。設定ファイルの `output.mask_mode = "plain"` は終了コード2で拒否します。

**旧0.21.0-rc.1の注意:** ZIPのtable形式は `--mask-mode redact` でも生の一致値を表示します。公開済み0.21.0-rc.4では修正済みです。rc.1の既存ZIPは変更していないため、rc.4を取得してください。

`--fail-on-findings` は指定値以上で失敗します。「1件以上」は `1` です。`0` は無効な入力として終了コード2になります。

終了コード: `0` は指定した失敗条件に該当しない、`1` は検出数・重大度等の条件に該当、`2` は実行エラーやスキャンのファイル数上限による打ち切りです。
通常の非対話ファイルスキャンでは、検出数・ファイル数・テキストのサイズ上限、読込み/UTF-8エラーも終了コード2です。baselineで既知の検知を抑制しても、収集時の上限到達を解除しません。サマリの `limit_reached` / `file_limit_reached` / `max_file_size_reached` / `read_error_reached` を確認してください。無視・バイナリのスキップがあるため、終了コード0だけでは全件スキャンを保証しません。

## よく使う操作

```bash
veil init --non-interactive       # 任意: veil.toml を生成
veil rules list                  # 有効なローカルルール
veil rules explain creds.aws.access_key_id
veil config check
veil doctor
```

既存検出を [baseline](docs/baseline/usage.md) に保存して段階的に導入できます。検知数・ファイル数・サイズ上限または読込みエラーがある作成は終了コード2で拒否し、既存baselineを保全します。秘密の生値の変更を識別する仕組みではありません。リスク承認や修正完了の証明には使わないでください。
`init` で作る設定はスコアによる失敗条件等を追加します。生成内容を確認してください。
`veil filter` は標準入力をマスキングします。ファイル書換えの `veil mask` は、まず `--dry-run` で確認してください。
各操作の詳細は `veil <command> --help` を参照してください。

## ローカルルールと移行

カスタムルールは `core.rules_dir` でローカルのRulePackを指定します。
旧設定 `core.remote_rules_url` / `VEIL_REMOTE_RULES_URL` は移行エラーになります。設定を取り除き、必要なルールをローカルで用意してください。
通信失敗によってルールが減ったままスキャンを続ける動作はありません。

署名付きRulePackを使う場合:

```bash
veil update --local-pack rules.zip --public-key publisher.hex
```

`rules.zip` に対応する `rules.sig` と明示した信頼鍵が必要です。署名と有効期限を検証します。
既存の `.veil/rules` は上書き・削除しません。別のルールディレクトリを設定するか、新しいプロジェクトで適用してください。
試用版の更新上限はZIP全体16 MiB、展開後の各ファイル4 MiB、合計64 MiB、4096エントリーです。署名が正しくても超過したPackは終了コード2で拒否します。

更新は署名・期限・ルール内容を検証して配置します。期限が明記されたPackは、期限後のスキャンやルール一覧でも読み込みを拒否します。自動では有効化せず、既存設定も書き換えません。使用する設定ファイルの既存 `[core]` 節に次を追加し、`veil rules list`（別の設定ファイルなら `veil --config <path> rules list`）で追加ルールを確認してください。

```toml
[core]
rules_dir = ".veil/rules"
```

組み込みルールの利用に `veil update` は不要です。

## 開発構成と既存verify

通常の `cargo build` / `cargo test` は `veil-config`・`veil-core`・`veil-cli` を対象にします。
既存のEvidence Pack v1を検査する `veil verify` は通常CLIと最小構成の両方で利用できます。ハッシュ・外部run_meta anchor・完全性・入力上限を確認し、既存の入力契約を維持します。
新規の台帳・Detect/Prove/Decide・PatchGate/VEX・署名v2契約は公開済み0.21.0-rc.4に含みません。
`veil-pro`・`veil-server`・`veil-guardian` は保全した旧メンバーで、通常CLIの依存には含みません。物理削除や公開リリースは未実施です。

ライセンス文書: [LICENSE-MIT](LICENSE-MIT)、[LICENSE-APACHE](LICENSE-APACHE)。
配布物の第三者ライセンス・著作権表記と同梱ソースは [THIRD_PARTY_NOTICES](THIRD_PARTY_NOTICES.md) と [NOTICE](NOTICE) を参照してください。


## 現行mainから維持した操作（2026-10-13）

`scan --preset fintech-jp` 等の組み込みpreset、`scan --interactive`、`rules promote-templates`、設定の継承・入力エラー検出を維持しています。`logs-jp` は `veil init --preset logs-jp` でローカルのlogルールを準備してから使用します。詳細な選択肢は各コマンドの `--help` を参照してください。
部分マスクは8文字以下を全て隠し、9文字以上は末尾4文字だけを表示します。先頭4文字は表示しません。

`veil lsp --preset fintech-jp` は独立した `veil-lsp` を標準入出力で起動します。CLI配布ZIPにはLSP本体を同梱しません。ソースから `cargo install --locked --path crates/veil-lsp` で導入し、PATHへ追加してください。設定・presetはCLIから子プロセスへ渡します。通常スキャンはLSPを起動せず、LSPのサーバー依存をリンクしません。

## RC作成時の検証記録

以下は配布対象の判断に使った履歴です。公開済みRCの取得は冒頭の手順を使ってください。

2026-10-13: 0.21.0-rc.1候補で、baseline適用による上限到達情報の消失と、Gitの最初のコミットの未スキャンを修正しました。修正後のLinux x86_64 musl静的候補は、Ubuntu 24.04.5 LTS（Linux 7.0.0-38-generic）の実機でビルド・起動・配布検査を確認しました。旧ARM64・QEMU・Windows候補の結果は現行候補へ流用せず、Windowsネイティブ／MSVCは未検証です。対話モードの不完全判定とGit blob欠損時の保存拒否も修正し、macOSでの回帰とLinuxネイティブでの全構成・配布物の再検証を確認しました。[リリース条件](docs/ci/oss-cli-release.md) に検証範囲を記載しています。
