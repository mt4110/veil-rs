# Linux配布方式と対応環境

2026-10-13: 0.21.0-rc.1のLinux x86_64 musl静的候補は、Ubuntu 24.04.5 LTS（Linux 7.0.0-38-generic）のx86_64実機でビルド・起動・配布検査を確認しました。以下の旧ARM64・QEMU・Windows候補の記録は統合前の結果で、現行候補へ流用しません。Windowsネイティブ／MSVCは未検証です。

Status: `reference`
Updated: 2026-10-14

[v0.21.0-rc.1](https://github.com/mt4110/veil-rs/releases/tag/v0.21.0-rc.1) は公開済みプレリリース。Linux版はx86_64 muslのZIPを配布している。初回の取得・チェックサム確認・自分のコードのスキャンは [チュートリアル](getting-started.md#ubuntuで自分のコードを試す) を参照する。同梱README・NOTICE・第三者表記・SHA256SUMSを保持する。
以下の候補作成・検証記録は全Linux環境の対応を意味しない。旧ARM64候補は今回の配布対象ではない。リポジトリの後続文書修正は既存配布ZIPへ自動反映されない。

## 選定した方式と最低条件

Linux向けの配布候補はmusl静的リンクを優先する。GNUビルドはソースからのローカル利用・診断用として保全する。

| 方式 | 必要条件 | 実行確認 | 未検証・制限 |
| --- | --- | --- | --- |
| 旧ARM64 musl静的候補（統合前） | 64bit・little endian ARMv8-AのLinux。共有libc、libz、libgccの追加インストール不要 | Ubuntu 22.04.5 / Alpine 3.22.6のARM64コンテナ。両方ともLinux 6.8.0-90-generic | 最低カーネル版は未確定。旧カーネル、他CPU、実機/別VMでの動作は未検証 |
| ARM64 GNU候補（前回のDebian 13ビルド） | glibc 2.39以上、libz.so.1、libgcc_s.so.1、標準libm/libc/ローダー | Debian 13 ARM64で起動 | Ubuntu 22.04のglibc 2.35では起動失敗。glibc 2.39より古い環境に配布しない |
| x86_64 musl静的候補（0.21.0-rc.1） | x86_64専用の64bit Linuxバイナリ。共有libc/libz/libgcc不要 | Ubuntu 24.04.5 LTS / Linux 7.0.0-38-genericのx86_64実機でネイティブビルド・起動・マスキング・終了コード0/1/2・配布検査を確認 | 最低カーネル版、旧カーネル、他ディストリビューション・他CPU・性能は未検証 |

静的候補はRustやmuslを利用者の端末へインストールせずに起動できる。ファイルの読み取り権限と、一時処理・明示した出力先への書込み権限は必要。
カーネル下限を検証できていないため、上記より古いカーネルでの対応を約束しない。コンテナ試験は同じホストカーネルを共有しており、2種類のカーネルを試したものではない。
macOS・Windowsには各OS用の実行ファイルが必要。Linux ELFをそのまま起動できない。Windowsの現行配布候補は実起動未検証。

GNU候補のGLIBC_2.39参照はRustのプロセス起動に使われるweakなpidfd_spawnp / pidfd_getpidだったが、リンク時のversion requirementにより旧glibcのローダーで実際に拒否された。
シンボル名だけを書き換えたり、チェックを無効にして最低条件を下げたりしない。古いglibc向けGNUビルドが必要なら古いsysroot/ビルド環境から再作成し、別の候補として受け入れる。今回は実施していない。

## ARM64上での静的候補の作成

以下はLinux ARM64上で検証した構成。macOS上やx86_64からARM64へ渡すクロスコンパイルの手順ではない。
Rust 1.92.0、同じCPU向けのmusl Cツール（Debian/Ubuntuのmusl-tools）、binutilsのreadelf、Python 3.12以上、Gitが必要。
MSRVのRust標準ライブラリに同梱されたmusl 1.2.3とLLVM libunwindを使用する。Rust側のmusl構築・修正は [1.92.0の公式スクリプト](https://github.com/rust-lang/rust/blob/1.92.0/src/ci/docker/scripts/musl.sh) を参照。

```bash
rustup target add --toolchain 1.92.0 aarch64-unknown-linux-musl
export CARGO_TARGET_AARCH64_UNKNOWN_LINUX_MUSL_LINKER=musl-gcc
export CC_aarch64_unknown_linux_musl=musl-gcc
export RUSTFLAGS='-C target-feature=+crt-static'
export ZLIB_STATIC=1
export LIBGIT2_NO_VENDOR=0
cargo +1.92.0 rustc --locked --release -p veil-cli --bin veil --target aarch64-unknown-linux-musl -- -C "link-arg=-Wl,-Map,$PWD/veil-musl-link.map"
```

この設定はglibc用のCコンパイラによる混入や、環境のzlib/libgit2への動的参照を避ける。結果のELFで確認するまでは静的配布を成立した扱いにしない。
Rustup環境では、以下のPython検査も同じコンパイラを参照するよう `RUSTUP_TOOLCHAIN=1.92.0` を設定する。

```bash
export RUSTUP_TOOLCHAIN=1.92.0
python3 scripts/check_cli_release.py --binary target/aarch64-unknown-linux-musl/release/veil --bundle <新しい候補ディレクトリ> --archive <新しいZIP>
python3 scripts/verify_cli_archive.py --archive <新しいZIP> --require-static --link-map veil-musl-link.map --execution-mode native --execution-host-architecture aarch64 --report <新しいJSON記録>
```

`--require-static` はLinux上で、ELFのPT_INTERP、DT_NEEDED、GLIBC参照が残る候補を拒否する。静的ELFであることだけではCPU・カーネル・全機能の互換性まで証明できない。
実際に展開したCLIのhelp・マスキングJSON・終了コード0/1/2・原文/ソース/checksum/リンクも検査する。
静的CRTの実装・版はELFだけから断定せず、指定target、コンパイラ、リンクマップを含めて記録する。
同じ候補のリンクマップにRust self-containedのlibc/libunwindがなく、または未確認のGNU libgccアーカイブがコードを提供した場合は成功記録を作らず、表記を再確認する。

## 表記と確認範囲

[第三者表記](../../THIRD_PARTY_NOTICES.md) と [NOTICE](../../NOTICE) は、muslとLLVM libunwindの原文も同梱する。
ARM64・x86_64のリンクマップではRustのself-contained libc.aとlibunwind.aを確認し、GNU libgccアーカイブの取り込みがないことも確認した。
別コンパイラ・別CPUで静的ランタイムが変われば再確認する。静的配布用の表記ゲートは今回確認したRust 1.92.0に限定する。

現在のCIにx86_64 muslのビルド・静的検査を追加した。現行0.21.0-rc.1はx86_64実機上で検証した。head `5cf3d1e` の[GitHub-hosted CI](https://github.com/mt4110/veil-rs/actions/runs/38400609392)でもmuslのビルド・展開アーカイブ検査が成功。後続文書headは別途確認する。
配布用ZIP・バイナリは公開/自動アップロードせず、既存CIは検証記録だけを保存する。
詳細なゲートは [OSS CLIリリース条件](../ci/oss-cli-release.md) を参照。

## 旧x86_64候補のビルドと実行方式（2026-10-12、統合前）

今回のビルドはARM64ネイティブのRust 1.92.0から `x86_64-unknown-linux-musl` へのクロスコンパイル。
C側はARM64上で動くx86_64 GNU cross GCCにx86_64用musl-gcc.specsを指定し、Rust self-containedの静的CRTへリンクした。GNU libgccコードの取り込みはない。
ELF machineはAdvanced Micro Devices X86-64。PT_INTERP / DT_NEEDED / GLIBC参照はなく、同梱NOTICE・原文・ソース・checksum・相対リンクを検査する。

起動はARM64のLinux VMに登録された `/usr/bin/qemu-x86_64`（binfmt_misc、POCFフラグ）経由。QEMUの版は未取得。
amd64コンテナ内のunameがx86_64を返しても、物理/実行ホストがx86_64である証拠にはしない。実際のホストCPUはエミュレータの外側で確認したaarch64。
UbuntuとAlpineの試験は同じ6.8.0-90-genericホストカーネルを共有する。

検証記録は `execution.mode`、申告した実ホストCPU、runtimeが報告するCPU、ELFのCPU、エミュレータ名を分けて保持する。
指定を省略した場合はunknown。native指定ではホスト/runtime/ELFの既知CPUが一致し、emulatorが指定されていないことを確認する。
実行方式は呼び出し元の申告で、あらゆる仮想化を自動検出する機能ではない。ホスト/登録情報の証拠と一緒に読む。

```bash
python3 scripts/verify_cli_archive.py --archive <x86_64候補ZIP> --require-static --link-map <同じビルドのリンクマップ> --execution-mode emulated --execution-host-architecture aarch64 --emulator 'QEMU user-mode via binfmt_misc; version unknown' --report <新しいJSON記録>
```

ネイティブx86_64ホストで受け入れる際は実ホストを外側から確認し、native / x86_64を申告して同じ検査を行う。エミュレーションの成功記録をその代わりにしない。

## 現行RCのx86_64ネイティブ検証（2026-10-13）

0.21.0-rc.1をUbuntu 24.04.5 LTS / Linux 7.0.0-38-genericのx86_64実機でビルド・実行した。CPUエミュレーションは使用していない。
Rust 1.92.0（host x86_64-unknown-linux-gnu）、target x86_64-unknown-linux-musl、GCC 13.3.0、Python 3.12.3を使用。
musl-tools/dev 1.2.4-2を検証領域へ展開し、musl-gcc.specsのinclude/libパスだけをその領域に合わせた。システムへのインストールは行っていない。
リンクマップで最終ランタイムはRust self-contained musl 1.2.3 / LLVM libunwind、GNU libgccアーカイブのコード取り込みなしを確認。musl 1.2.4の原文COPYRIGHTも同梱1.2.3の原文とバイト一致した。

x86_64 Linux上での標準musl-tools構成では、上のARM64手順のtargetと環境変数を以下へ読み替える。

```bash
rustup target add --toolchain 1.92.0 x86_64-unknown-linux-musl
export CARGO_TARGET_X86_64_UNKNOWN_LINUX_MUSL_LINKER=musl-gcc
export CC_x86_64_unknown_linux_musl=musl-gcc
export RUSTFLAGS='-C target-feature=+crt-static'
export ZLIB_STATIC=1 LIBGIT2_NO_VENDOR=0 RUSTUP_TOOLCHAIN=1.92.0
cargo +1.92.0 rustc --locked --release -p veil-cli --bin veil --target x86_64-unknown-linux-musl -- -C "link-arg=-Wl,-Map,$PWD/veil-musl-link.map"
python3 scripts/check_cli_release.py --binary target/x86_64-unknown-linux-musl/release/veil --bundle <新しい候補ディレクトリ> --archive <新しいZIP>
python3 scripts/verify_cli_archive.py --archive <同じZIP> --require-static --link-map veil-musl-link.map --execution-mode native --execution-host-architecture x86_64 --report <新しいJSON記録>
```

通常scan/verifyの終了コード0/1/2・マスキング、baseline上限・CI取得元・既存v1 verify・安全境界・libgit2版の回帰をネイティブで確認した。
対話モードの不完全判定とGit blob取得失敗時の保存拒否も含む0.21.0-rc.1のmusl targetで、通常357件・最小構成356件・JP358件（それぞれCLI/core/config）、独立LSP25件の全テストを確認。`--include-ignored` で1,000件のテンプレート読込みも実行し、失敗・ignoredはいずれも0件。構成間には重複テストがあるため合計を固有件数とはしない。
release構成の修正回帰・baseline上限・既存v1 verify/safetyの84件も成功。通常・最小・LSPのClippy全target警告検査、書式・構造・metadata・文書・Python85件も成功。
検証コードはベースcommit `aa67bc4c34029a6745deee51e39c7b163c75dd7e` に追加修正を適用したもので、ローカル修正版とLinux側2,950ファイルのハッシュ一致を確認した。既存verifyは変更しない。文書更新による再固定では、本番コード・テスト・lock・ツールチェーンを変えず、同じ検証済みバイナリで新しい文書を同梱し直す。ソースと配布物のハッシュ、固定commit、検証記録は別の準備レポートへ記録する。
完成ZIPからCLIを取り出し、ビルド時と同じバイナリhashを確認した上で21ケースを実行した。対話モードの各不完全判定、CLI/config由来のbaseline抑制、対話と保存の同時指定、Git commit/履歴/stagedのblob欠損、staged HEAD/tree欠損で終了コード2・部分結果非出力・新規baseline非作成・既存baselineのバイト保全を確認した。正常な対話q終了と、最初のコミット前の正常なstaged保存も確認した。Gitlinkの明示的なスキップ、baseline適用時の上限保持、最初のGitコミットと`--since`の検査はrelease回帰にも含める。[公開前の必須課題](../ci/oss-cli-release.md#公開前の必須課題) に残る範囲を記録する。
release CLIからのpreset付きLSP起動とrelease LSP単体起動をstdioで確認。initializeの版表示0.21.0-rc.1、マスキング診断、Mask value修正候補、didChange/didCloseの診断更新、shutdown/exit 0を確認した。VS Code等のGUIクライアント実操作は未検証。
CLI ZIPにはLSP本体を同梱しない。LSPバイナリを配布対象へ加える場合は、独自の依存・原文ライセンス・同梱packetを別途確認する。
ZIPの564エントリーは許可ファイル一覧と完全一致し、文書リンク・原文/ソース/checksumを検査。第三者依存270件・ライセンスpayload 538件・native component 4件を照合した。ELFのインタープリター・NEEDED共有ライブラリ・GLIBC版付きシンボルはいずれもない。Windowsネイティブ/MSVC、現行RCのARM64/Alpine/旧カーネル、性能は未検証。Hosted CIは上記headで成功。
