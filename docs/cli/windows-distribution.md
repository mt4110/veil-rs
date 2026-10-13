# Windows配布候補

2026-10-13: 0.21.0-rc.1のLinux x86_64 musl静的候補は、Ubuntu 24.04.5 LTS（Linux 7.0.0-38-generic）のx86_64実機でビルド・起動・配布検査を確認しました。以下の旧ARM64・QEMU・Windows候補の記録は統合前の結果で、現行候補へ流用しません。Windowsネイティブ／MSVCは未検証です。

Status: `reference`
Updated: 2026-10-12

## 対応対象と検証の区別

現在のローカル候補は **x86_64-pc-windows-gnu / Rust 1.92.0**。ARM64 Linux上のMinGW-w64 12.0.0、GCC 14.2.0 POSIX、binutils 2.44でクロスビルドしたPE32+コンソール実行ファイルです。
[同じRust版の対応表](https://github.com/rust-lang/rust/blob/1.92.0/src/doc/rustc/src/platform-support.md)に従い、対象は64bit x86 Windows 10以降 / Windows Server 2016以降とします。この最低条件を実Windowsで確認したわけではありません。

| 項目 | 現在の証拠 |
| --- | --- |
| GNU候補の実ビルド | ARM64 Linuxからx86_64 Windowsへ成功。Cargo.lock固定、通常OSSプロファイル。 |
| 起動・CLIスキャン | Debian 13 amd64コンテナのWine 10.0（10.0~repack-6）＋QEMU user-modeで成功。実CPUホストはARM64。 |
| バージョン・help・JSON | veil 0.17.0、通常help、veil-v1、マスキング、終了コード0/1/2を確認。 |
| ZIP・同梱内容 | 展開後に同じCLIを実行し、allowlist、SHA256SUMS、原文/ソース、文書リンクを検査。 |
| Windowsネイティブ | 実機・Windows VMとも未検証。 |
| MSVC候補 | 既存Windows CIはMSVC用。現在の未コミット候補のビルド・起動結果は未取得。GNU候補の証拠を流用しない。 |

[Wine](https://www.winehq.org/about)はWindows APIの互換実装で、CPUエミュレータとは区別します。今回のCPU実行は別途QEMUに依存しています。binfmt_miscのqemu-x86_64 / POCF登録を確認しましたが、保持されたQEMU実行ファイルの版は未取得です。検証コンテナはnetwork=noneで起動し、作業専用のWine prefixを使用しました。

## 必要なDLL

今回のPE通常importは以下の12件。遅延importは0件です。

- advapi32.dll
- api-ms-win-core-synch-l1-2-0.dll（Windows API-set）
- bcryptprimitives.dll
- kernel32.dll
- msvcrt.dll
- ntdll.dll
- ole32.dll
- secur32.dll
- shell32.dll
- user32.dll
- userenv.dll
- ws2_32.dll

これらは対象Windowsが提供する依存で、ZIPには同梱しません。BCryptPrimitives.dllの由来は[MicrosoftのProcessPrng仕様](https://learn.microsoft.com/en-us/windows/win32/seccng/processprng)を参照してください。
PE importにはlibgcc_s_seh-1.dll、libwinpthread-1.dll、VCRUNTIME140.dllなどの追加ランタイムを検出しませんでした。RustやMinGWの利用者側インストールは不要とする候補です。
この検査はPEの通常/遅延importに限定され、実行時LoadLibrary、Windowsの推移的DLL解決、クリーンなWindowsでの動作保証にはなりません。DLLを任意のダウンロードサイトから追加する手順は提供しません。

## 同梱するランタイム表記

リンクマップでMinGWのlibmingw32.a / libmingwex.a、GCCのlibgcc.a / libgcc_eh.aのオブジェクト寄与を確認しました。DLL依存がないことと、静的コードのライセンスが不要なことを混同しません。
[第三者表記](../../THIRD_PARTY_NOTICES.md)と[manifest](../../third_party/manifest.json)には、MinGW COPYING / AUTHORS / 実際のDebianパッケージのcopyright、GCC GPLv3 / Runtime Library Exception 3.1 / 実際のパッケージのcopyrightを原文のまま収録します。
Debian GCCパッケージのcopyrightには旧版名への参照も残っています。これを編集して現在版の著作権表記に見せず、GCC 14.2.0の上流GPL/例外文書と、実ツール版・パッケージSHAを併記します。法的な最終判断や全ターゲットのランタイム監査完了を意味しません。
Wine、QEMU、ビルドツールの実行ファイル、Windows DLL、PDB、private資料は配布しません。Cargoライセンス原文、colored/libgit2-sysソース、Rust標準ライブラリ表記も保全します。

## 起動と検査

Windows PowerShellでは、ZIPを新規ディレクトリへ展開して起動します。

```powershell
.\veil.exe --version
.\veil.exe --help
.\veil.exe scan . --format json --mask-mode redact
```

検証用ソースでは `scripts/check_cli_release.py` と `scripts/verify_cli_archive.py` がPEを直接読み、非標準DLL・遅延DLL・壊れたPEを拒否します。OS DLLの新しい依存が現れた場合は、その由来を確認してから一覧を更新します。
既存Windows CIもこの検査を使い、PEのCPU、DLL名、実行方式をJSONへ残します。現行候補をリモートへ送っていないため、CIの成功は未取得です。MSVCで追加ランタイムが必要になった場合はCIを失敗させ、配布方式・前提とライセンスを別途確認します。

Linux上でWineを使う場合は、`verify_cli_archive.py` に `--binary-name veil.exe --wine-runner <Wine実行ファイル> --execution-mode emulated --execution-host-architecture <実CPU> --emulator <CPUエミュレータ>` を明示します。今回のGNU候補では `--windows-gnu-link-map <同じ候補のリンクマップ>` も確認します。native指定でWineを成功記録にすることは拒否します。
この経路はWine＋CPUエミュレーション用です。CPUをエミュレーションしないWineのみの証拠は未検証で、今回の方式として分類しません。

未検証: Windows 10/11/Serverでの実起動、MSVC、Windows ARM64/32bit、クリーン環境、長いパス・Windows固有ACL/端末表示・署名/SmartScreen、旧Windows、性能、現行GitHub-hosted CI。公開は行っていません。

## ネイティブWindows / MSVC環境の確認（2026-10-12、GNU候補の後続確認）

確認したローカルVMの管理ツール・既知の保存先・稼働ゲスト・設定済みの接続経路では、利用できるWindows OSを特定できませんでした。macOS ARM64とColimaのLinux ARM64が現在利用できる環境です。全端末や全組織の環境が存在しないことを証明する確認ではありません。
リポジトリのself-hosted runner一覧は0件。GitHub-hosted `windows-latest` の既存CI定義はありますが、現在の未コミット候補を送信・起動していないため、その実行結果は未取得です。TailscaleアプリのbackendはStoppedで、Windows peerの有無は確認できませんでした。サービス起動や接続設定は変更していません。

ローカルのHomebrew LLVM 23.1.1に `clang-cl` はあります。Rustの既定rustupツールチェーンにはWindows MSVC標準ライブラリもありますが、これはSDKやCRTの代替にはなりません。Rust 1.92.0のインストール済みtargetはmacOS ARM64のみでした。
`x86_64-pc-windows-msvc` を明示した最小C入力で、Windows SDKの `windows.h` とCRTの `stdio.h` が見つからないことを確認しました。MSVC用のlinkerも利用できません。これはヘッダー可用性の検査であり、Veil候補のビルド試験ではありません。

そのため今回のMSVC候補は **未生成**。MSVC実起動・DLL一覧・同梱内容の実検査は未検証のままです。前回のGNU候補の12 DLLをMSVCのDLL一覧として流用しません。前回のGNU ZIPと成功記録も上書きしていません。この後続文書変更を前回ZIPに同梱済みとして扱いません。
[Rust 1.92.0のMSVC資料](https://github.com/rust-lang/rust/blob/1.92.0/src/doc/rustc/src/platform-support/windows-msvc.md)では、非WindowsからのMSVCクロスビルドはサポート対象外です。SDK・ツールの新規導入やWindows VM作成は今回行っていません。

## Windows環境を用意した後の検証手順（未実行）

以下は64bit x86 Windows 10以降 / Server 2016以降で、現在の候補ソースを検証するための手順です。MSVCとWindows SDKを含むC++ Build Tools、Rust 1.92.0のMSVCツールチェーン、Python 3.12以上が必要です。[Microsoftの案内](https://learn.microsoft.com/en-us/cpp/build/building-on-the-command-line?view=msvc-170)に従い、x64用Developer PowerShellを使用してください。管理者権限を前提にしません。
現在の公開タグを取得するだけでは未コミット候補の検証になりません。レビュー対象の現在のソースを用い、ライセンス原文の改行は `.gitattributes` に従って保全します。

PowerShellでは外部コマンドの非ゼロ終了を明示的に止めます。下記はソースrootで実行し、出力は新規の一時ディレクトリへ作ります。ビルド後は同じコンパイラを検査スクリプトが参照します。

```powershell
$ErrorActionPreference = 'Stop'
if ([Environment]::OSVersion.Platform -ne 'Win32NT' -or
    [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString() -ne 'X64') {
    throw 'This procedure requires x64 Windows; ARM64/x64 emulation is a separate check.'
}
function Invoke-Checked {
    param([string]$Command, [string[]]$Arguments)
    & $Command @Arguments
    if ($LASTEXITCODE -ne 0) { throw "$Command failed: $LASTEXITCODE" }
}
$rustupBin = Join-Path $env:USERPROFILE '.cargo\bin'
if (-not (Test-Path (Join-Path $rustupBin 'rustc.exe'))) {
    throw 'Rustup rustc.exe is required.'
}
$env:PATH = "$rustupBin;$env:PATH"
$env:RUSTUP_TOOLCHAIN = '1.92.0-x86_64-pc-windows-msvc'
$env:LIBGIT2_NO_VENDOR = '0'
$env:PYTHONDONTWRITEBYTECODE = '1'
Get-Command cl.exe, link.exe, cargo.exe, rustc.exe, python.exe -ErrorAction Stop
$checkRoot = Join-Path ([IO.Path]::GetTempPath()) ('veil-msvc-' + [guid]::NewGuid())
New-Item -ItemType Directory -Path $checkRoot -ErrorAction Stop | Out-Null
$env:CARGO_TARGET_DIR = Join-Path $checkRoot 'target'
Invoke-Checked 'rustc' @('-vV')
Invoke-Checked 'cargo' @('fetch', '--locked')
Invoke-Checked 'cargo' @('build', '--locked', '--offline', '--release', '-p', 'veil-cli', '--bin', 'veil', '--target', 'x86_64-pc-windows-msvc')
$binary = Join-Path $env:CARGO_TARGET_DIR 'x86_64-pc-windows-msvc\release\veil.exe'
$bundle = Join-Path $checkRoot 'bundle'
$archive = Join-Path $checkRoot 'candidate.zip'
$record = Join-Path $checkRoot 'archive-check.json'
Invoke-Checked 'python' @('scripts/check_cli_release.py', '--binary', $binary, '--bundle', $bundle, '--archive', $archive)
Invoke-Checked 'python' @('scripts/verify_cli_archive.py', '--archive', $archive, '--binary-name', 'veil.exe', '--report', $record, '--execution-mode', 'native', '--execution-host-architecture', 'x86_64')
Write-Output $checkRoot
```

このPowerShell手順自体もWindowsでは未実行です。環境とコンパイラを確認し、ビルドとZIP展開後の起動・マスキング・終了コード0/1/2・通常/遅延DLL・原文/ソース・リンクを検査するために用意しました。非標準DLLが出た場合は検査を止め、ランタイムの配布条件と表記を確認してください。許可リストを弱めて通さないでください。
結果JSONのWindows OS、native実行方式、PE x86_64、rustcのMSVC host、source fingerprint、ZIP/EXE checksumを照合します。DLL import検査だけで実行時LoadLibraryやクリーンWindowsの推移依存が確認できたとは扱いません。Rust更新・静的MSVC CRTへ変更する場合には、現在のライセンスpacketに加えて実際に使用するMSVCランタイムの表記を改めて確認します。

[チュートリアル](getting-started.md)、[利用ガイドライン](usage-guidelines.md)、[リリース条件](../ci/oss-cli-release.md)も参照してください。
