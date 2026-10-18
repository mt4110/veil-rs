# OSS CLI release checks

Status: `reference`
Updated: 2026-10-18

## 正式版0.22.1の公開状態（2026-10-18）

[0.22.1](https://github.com/mt4110/veil-rs/releases/tag/v0.22.1)は公開済み正式版で、Latestです。氏名・顧客／会員IDの誤検出2件の限定修正を含みます。通常の個人情報、同じ行の別ID、テスト用IPv4の検出を維持しています。

- 署名タグの配布元はPR #173マージ後main `62b713cccf23b02eb4210d743dce12b0b7f1186f`。[配布元main CI](https://github.com/mt4110/veil-rs/actions/runs/39083584473)は全6件成功。これは配布元の記録で、後続の文書更新や現行headの検証結果ではありません。
- 固定ソースからRust1.92.0でLinux x86_64 musl・macOS ARM64をネイティブ生成し、ZIP展開後のCLI・秘匿・終了コード0/1/2・Gitステージ点検・導入時の保全・日英操作導線・第三者表記・内部チェックサム・今回の修正を検証済みです。
- 署名タグをローカルとpush後に検証し、GitHub側の署名検証と配布元一致も確認済み。タグメッセージには両ZIPのSHA-256を含みます。ZIPごとの独立署名は添付していません。SHA256SUMS単独は発行者の真正性を証明しません。
- 公開後、ZIP2種とSHA256SUMSを認証なしで再取得し、検証済み成果物とのバイト単位の一致を確認済みです。公開済みZIPには準備中の文書が残るため、取得には[最新チュートリアル](../cli/getting-started.md#ubuntuで自分のコードを試す)を使用します。後続の文書更新は既存ZIPを差し替えません。
- macOSはad-hoc署名の試用版。Developer ID署名・公証は未実施で、ダウンロード後のGatekeeperは未検証です。Windows・Intel Mac・Linux ARM64・LSP単体配布・crates.io公開は対象外です。
- 実作業の限定的な自己利用・合成検査と、独立利用者の継続利用は区別します。

## 0.22.0の公開記録

[0.22.0](https://github.com/mt4110/veil-rs/releases/tag/v0.22.0)も公開済み正式版として保持しています。Rust不要の導入スクリプト、日英help・チュートリアルを含みます。署名タグの配布元は `5de440e` で、公開時のmain `f995c70` とtreeが一致し、[当時のmain CI](https://github.com/mt4110/veil-rs/actions/runs/38843403179)も成功しました。タグ・Release・ZIP・SHA256SUMSは変更していません。

## 正式版0.21.0の公開状態（2026-10-16）

- [v0.21.0](https://github.com/mt4110/veil-rs/releases/tag/v0.21.0)は公開済み正式版（公開時点ではLatest）。公開元時点のmanifest・CLI/core/config/LSP・Cargo.lock・Nixの版番号は0.21.0。rc.4から製品コード・外部依存の変更はない。
- 配布元・署名タグの参照先はPR #167マージ後main `61fda637a63993349a73f7256e4d38108e1258d2`。[配布元CI](https://github.com/mt4110/veil-rs/actions/runs/38824887889)はUbuntu/macOS・musl・MSRV・ガード2件の全6ジョブ成功。これは配布元の記録であり、後続の文書更新や現在のmainのCI結果を示さない。
- Linux x86_64 musl・macOS ARM64の最終ZIPをRust 1.92.0でネイティブ生成し、展開CLIの動作・秘匿・異常系・Gitステージ/フック・ライセンス・内部チェックサム・Linux静的リンクを確認した。ZIP2種とSHA256SUMSを公開後に認証なしで再取得し、検証済み成果物との完全一致を確認した。
- 署名タグはローカル・push後の参照先一致・GitHubの署名検証を確認済み。タグメッセージには両ZIPのSHA-256を含む。Linux ZIP用の独立した署名ファイルは添付しない。SHA256SUMS単独は発行者の真正性を証明しない。
- macOSはad-hoc署名の試用版。Developer ID署名・公証・ダウンロード後のGatekeeperは未検証。Windows、Intel Mac、Linux ARM64、LSP単体配布とcrates.io公開は含めない。
- 公開済み正式版とrc.4のタグ・ZIP・SHA256SUMSは後続の文書変更で差し替えない。正式版ZIP内には作成時点の準備中表記・rc.4取得手順が残るため、取得には[最新版チュートリアル](../cli/getting-started.md#ubuntuで自分のコードを試す)を使う。
- 実作業での継続利用は未確認の追跡項目として保持する。合成検査と独立利用実績を同一視せず、技術的な公開完了とは分ける。

## M1入りrc.3の準備記録（公開済み・履歴）

main `bded3ae` は別インデックスの検査・生成フックの停止条件・既存フック保全を修正し、[マージ後CI](https://github.com/mt4110/veil-rs/actions/runs/38670786876)全6ジョブが成功した。
旧候補04745e7のZIP・本文・検証記録を保全し、新しい固定コミットからLinux x86_64 musl・macOS ARM64のZIP2種とSHA256SUMSを再生成・検証して、同じrc.3ドラフトへ反映する。この準備時点では公開済みrc.2を変更せず、rc.4は作らない方針だった。その後rc.3を公開し、PR #163の氏名ルール修正に伴いrc.4を別候補として準備する。
最終ZIPから取り出したCLIで、既存の安全性検査に加え、別index、実際の部分コミット、フック停止と既存フック保全を両OSで確認する。macOSはad-hocのみでDeveloper ID・公証・隔離属性付きGatekeeperは未検証。
上記はrc.3準備時の記録で、現在rc.3は公開済み。rc.4準備時も配布元コミットとハッシュを準備記録とドラフト本文へ記録し、当時はタグpush・公開を行わなかった。現在の公開状態は冒頭の正式版節を参照する。

## 公開対象と現在の制限

対象はconfig/core/cli、既存Evidence Pack v1 verify、ローカルのルールとGit、チュートリアルと利用ガイド、依存・梱包ガード。
新規監査機能・台帳・署名v2・PatchGate/VEX契約・未使用breaker・重複legacy入口・Proへの新規機能指定は含めない。main既存のverifyテストは削除しない。
旧メンバーと過去の公開文書は保全する。private・ローカル成果物は公開差分とZIPへ含めない。

以下はrc.1の公開履歴。manifest・CLI・独立LSP・Nix CLI packageを当時0.21.0-rc.1へ統一した。公開tag v0.20.0と旧manifest0.17.0は過去の履歴として保全する。当時の試用版は0.21.0-rc.1。配布対象はLinux x86_64 musl・macOS ARM64に限定する。macOSはad-hoc署名で、Developer ID署名・公証済みではない。[tag v0.21.0-rc.1とGitHubプレリリース](https://github.com/mt4110/veil-rs/releases/tag/v0.21.0-rc.1) は公開済みで、検証済みZIP2種とZIP全体のSHA256SUMSを添付している。main統合コミットは `a090a3c871d166046e14df2f8f11214bc9dc4641`。今回の文書修正は既存のtag・Release・ZIPを変更しない。
通常CLIのライセンスpacketは対象normal/buildグラフ用。依存・lock・コンパイラ・配布方式を変更したら再照合する。LSP本体やMSVC runtimeを同じpacketで監査済みと扱わない。

## 0.21.0-rc.2公開済みプレリリース

main `5ae098face5f671efc106a2848f2eaf7d8ae3f79` の修正を基準に、対応4クレート・workspace依存・Cargo.lock・Nixの版番号をrc.2へ揃えた。PR #155〜#157の出力秘匿、テキスト入力/ignore/設定診断、設定とルール読込み、maskの改行/安全な書換え、ignore後の件数予算の修正を含む。
配布元 `449a1e2e` からLinux x86_64 musl・macOS ARM64を別々にビルドし、展開ZIPの起動・配布検査・回帰を確認した。PR #158マージ後main `b1d3234` とtreeが一致し、[main CI](https://github.com/mt4110/veil-rs/actions/runs/38544422228)の全6ジョブが成功。[rc.2のRelease](https://github.com/mt4110/veil-rs/releases/tag/v0.21.0-rc.2)にZIP2種とSHA256SUMSを公開済み。版番号だけから検証済みとは判断しない。

署名タグ `v0.21.0-rc.2` は検証済み配布元を指し、注記にZIPとSHA256SUMSのハッシュを含む。rc.2公開当時、安定版Latestはv0.20.0のままだった。今回の文書更新は公開済みRelease/ZIPを差し替えない。macOSはad-hoc署名の整合検査・ローカル実行のみで、Developer ID、公証、ダウンロード隔離属性付きのGatekeeper試験は提供しない。

## 自動CI

`ci.yml` がPR/main・develop push/手動実行の入口。`oss-profile.yml` を共通手順として呼ぶ。
MSRV1.92.0、x86_64 Linux musl、Rust 1.92.0固定のUbuntu/macOS nativeを定義する。Windowsは今回のCI・配布対象外。最新stableの検証とは扱わない。通常・最小構成、JPチェックサム、独立LSPを検査する。

- 全workspace書式、対応crate全targetのClippy（警告をエラー扱い）とテスト。
- 実CLIで対話プレビューの複数/反復一致の秘匿、引用文字列/別IDのignore非抑制、Git staged/commit/historyの不正UTF-8拒否とbaseline保全、TOML診断の生値非出力、maskの設定失敗時の入力/バックアップ保全を通常・最小構成で確認する。
- 実CLIで全スキャン系コマンドの明示設定欠損拒否・設定継承/カスタム/RulePack適用、Git履歴の不正UTF-8/上限拒否、maskの改行/権限保持・全入力事前確認・backup非上書き・symlink拒否を通常/最小構成で検査する。
- 既存verifyの構造・ハッシュ・anchor・完全性・ZIP上限と29件のmain回帰。
- アーキテクチャ・公開範囲・依存ライセンス・文書リンクとPythonガード。
- 実release CLIのversion/help、veil-v1マスキング、0/1/2、不要な状態保存の検査。
- text/JSON/HTML/Markdown（md別名含む）/tableで既定・redact・partialの秘匿、unsafe/plainのCLI明示生出力、設定plainの拒否（redact同時指定を含む）を実行検査する。tableのMatch列は秘匿時に `<REDACTED>`。公開済み0.21.0-rc.1 ZIPのtableには生値出力問題が残り、このソース修正だけで既存配布物は更新されない。
- 正確なallowlistでZIPを作り、展開後の起動・checksum・原文/ソース・相対リンクを検査。
- macOSの非標準動的依存、Linux静的ELFを検査。Windows import検査は旧候補用で、今回の対応を示さない。

自動artifactはログ・JSON検証記録・checksumだけ。バイナリ/ZIPは自動公開しない。PR #156マージ後のmain `79cf4d92` の[CI](https://github.com/mt4110/veil-rs/actions/runs/38528215294)は全6ジョブ成功。後続PRはそのheadで通常/最小構成・両OS・SOT guardを再確認する。
既存PR SOT guard、dogfood/release draftsの運用は保全。旧Go/Nix/Guardian/SQLx/UI検証は `legacy-checks.yml` の手動手順で、通常CLIの合格と区別する。

## ローカル検証

```bash
cargo fetch --locked
cargo fmt --all -- --check
cargo clippy --locked --offline -p veil-cli -p veil-core -p veil-config --all-targets -- -D warnings
cargo test --locked --offline -- --include-ignored
cargo clippy --locked --offline -p veil-cli -p veil-core -p veil-config --no-default-features --all-targets -- -D warnings
cargo test --locked --offline -p veil-cli -p veil-core -p veil-config --no-default-features -- --include-ignored
cargo test --locked --offline -p veil-cli -p veil-core -p veil-config --features jp_mynumber_checksum -- --include-ignored
cargo clippy --locked --offline -p veil-lsp --all-targets -- -D warnings
cargo test --locked --offline -p veil-lsp -- --include-ignored
python3 scripts/check_architecture.py
python3 scripts/check_cli_release.py
python3 -B -m unittest discover -s scripts -p 'test_*.py'
python3 scripts/check_docs_taxonomy.py
```

通常CLIはvendored libgit2修正版を使う。CIの `LIBGIT2_NO_VENDOR=0` と依存/実リンク検査を維持する。
最初のoffline検証には依存キャッシュが必要。workflow構文はactionlintで確認する。

## 配布候補

固定文書一覧は `scripts/check_cli_release.py` のDOCUMENTS。第三者ファイル一覧は [manifest](../../third_party/manifest.json)。
同梱範囲をディレクトリglobやリンク先の自動追加で広げない。private・余分なファイル・symlink・欠損リンク/anchor・checksum変更を拒否する。
既存候補ディレクトリ・ZIP・検証記録を上書きしない。

```bash
LIBGIT2_NO_VENDOR=0 cargo build --locked --offline --release -p veil-cli --bin veil
python3 scripts/check_cli_release.py --binary target/release/veil --bundle <新規候補dir> --archive <新規ZIP>
python3 scripts/verify_cli_archive.py --archive <新規ZIP> --execution-mode native --execution-host-architecture <実hostCPU> --report <新規JSON>
```

実行方式はnative/emulated/unknownを区別する。エミュレーション結果を実機互換性・性能の根拠にしない。HTTP/HTTPSリンクの到達性はofflineリンク検査の対象外。

## 未検証と公開前の判断

以下はrc.1作成時の判断記録であり、現行0.21.0正式版の残条件ではない。現行の未完了作業は冒頭の「正式版0.21.0の公開状態」を参照する。

baseline適用時の上限情報、Gitの最初のコミット、対話モードの不完全判定、Git blob取得失敗時の保存拒否を修正した0.21.0-rc.1のLinux x86_64 musl静的候補は、Ubuntu 24.04.5 LTS / Linux 7.0.0-38-genericの実機でネイティブビルド・実行・同梱検査を確認。同じmusl targetの通常357件・最小356件・JP358件・LSP25件、release関連回帰84件は失敗/ignored 0件。配布ZIPから取り出したCLIの21ケースで終了コード2・部分結果非出力・新規baseline非作成・既存baseline保全と正常系を確認した。release LSPのCLI経由・単体のstdio実起動、診断更新、修正候補、正常終了も確認した。構成間の件数は重複するため固有テスト総数として合算しない。
Windowsネイティブ/MSVC、現行RCのARM64/他ディストリビューション/旧カーネル、IDE GUI連携、LSPバイナリ配布のライセンスpacket、Developer ID署名/公証/ダウンロード後のGatekeeper/SmartScreenは未検証。
[Linuxガイド](../cli/linux-distribution.md)・[Windowsガイド](../cli/windows-distribution.md)の旧候補の結果を、現在のソースに流用しない。
Linux x86_64 musl・macOS ARM64用0.21.0-rc.1試用版として、同じ承認済みソースで最終ZIPを生成・検証する。macOSはad-hoc署名とし、Gatekeeper回避を通常のインストール手順にしない。旧tagへ今回の変更を含めた扱いにしない。
crates.io公開、旧UI/サービス全体の動作確認、性能の優位性、サイト・テレメトリ・DBはこの準備の合格条件へ追加しない。
今回の実行結果・main向け差分・保全hashは別の準備レポートへ記録する。公開や元ブランチへの適用の承認を代替しない。

## 公開前の必須課題

以下は0.21.0-rc.1作成時の検証・公開前ゲートの履歴。ここに残る再生成・公開前確認を現行の未完了条件として扱わない。

採用済みの修正と最新文書は隔離Gitブランチ・コミットに固定し、main向け差分の適用・再構成・公開範囲を確認する。ローカルの固定は公開・hosted CIの成功を意味しない。

追加の完全性判定を最小修正し、macOS ARM64のCLIで回帰を確認した。追加修正後のLinux x86_64でもmusl静的ビルド・全構成・配布物の再検証が完了した。Windowsネイティブ/MSVCは未検証のまま。

- 対話モード：上限・読取り異常の結果はレビューへ渡す前に終了コード2で拒否する。baselineによる全件抑制でも不完全性を解除しない。対話レビューとbaseline作成の同時指定は引数エラー2。
- Git読取り：commit/history/stagedのblob取得失敗をエラー2として伝え、部分結果を表示・保存しない。stagedのHEAD/tree/index取得失敗も正常な空結果へ変換しない。新規保存先の非作成と既存baselineのバイト列保全を検証する。Gitlinkはサブモジュールの参照として明示的にスキップし、blob欠損と区別する。

新しい回帰は24件。最初のstaged fixture作成でGit indexの参照検証に拒否されたケースはCLIの再現と数えず、削除を伴わない欠損参照のfixtureへ直して修正前・修正後を確認した。既存verifyのソース・期待値は変更しない。

修正後のLinuxネイティブ検証を終えた本番コード・テスト・lock・ツールチェーンを保ち、文書更新後の同梱物も再生成・検査する。公開までに、固定commitの既存hosted CI、対象OS/CPU、ad-hoc試用版の表記とZIP全体のSHA256を確認する。macOS最終配布物は今回の修正と最新文書に合わせた再生成・検査が必要。Windowsネイティブ/MSVCは未検証として保留し、現在のCLI ZIPへLSP本体を追加しない。

## 最終候補の保全

配布ZIP本体はCIの検査artifactに含まれない。最終文書コミットから両OSでreleaseビルドし、新規の候補保存先へZIP・検証JSON・ビルドログ・ZIP全体のSHA256SUMSを保存する。Git revision、実compiler、OS/CPUとネイティブ実行方式を記録する。既存候補は上書きしない。

macOSのad-hoc署名は開発者の身元確認や公証を提供しない。ローカルの署名整合・起動検査と、ダウンロード由来の隔離属性を伴うGatekeeper試験を区別する。後者は未検証として明記し、今回のローカル起動成功で置き換えない。

crates.io公開は今回の範囲外。Cargoのlicenseメタデータは4つのOSSクレートでMIT OR Apache-2.0を継承するが、ソースパッケージへのライセンス原文同梱はcrates.io公開前の別ゲート。

## 0.21.0-rc.4公開済みプレリリース（履歴）

[rc.4のRelease](https://github.com/mt4110/veil-rs/releases/tag/v0.21.0-rc.4)は公開済み。署名タグは配布元 `a897e8f` を指し、PR #164マージ後main `08538de` とtreeが一致する。[その時点のmain CI](https://github.com/mt4110/veil-rs/actions/runs/38695150522)は全6ジョブ成功。Linux x86_64 musl・macOS ARM64のZIP2種とSHA256SUMSを公開後に再取得して照合済み。PR #165マージ時点のmainは `dd86f92` だった。これは当時の記録であり、後続mainの状態を示さない。rc.4タグのソースは変わらない。macOSはad-hoc署名のみで、Developer ID・公証・ダウンロード時Gatekeeperは未確認。実作業での再利用も未確認。公開済みZIPの本文書変更による差し替えは行わない。
