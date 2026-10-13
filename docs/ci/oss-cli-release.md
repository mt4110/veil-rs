# OSS CLI release checks

Status: `reference`
Updated: 2026-10-13

## 公開対象と現在の制限

対象はconfig/core/cli、既存Evidence Pack v1 verify、ローカルのルールとGit、チュートリアルと利用ガイド、依存・梱包ガード。
新規監査機能・台帳・署名v2・PatchGate/VEX契約・未使用breaker・重複legacy入口・Proへの新規機能指定は含めない。main既存のverifyテストは削除しない。
旧メンバーと過去の公開文書は保全する。private・ローカル成果物は公開差分とZIPへ含めない。

この隔離候補のmanifest・CLI・独立LSP・Nix CLI packageは0.21.0-rc.1へ統一。公開tag v0.20.0と旧manifest0.17.0は過去の履歴として保全する。正式版案は0.21.0で、今回tag作成・公開は行わない。
通常CLIのライセンスpacketは対象normal/buildグラフ用。依存・lock・コンパイラ・配布方式を変更したら再照合する。LSP本体やMSVC runtimeを同じpacketで監査済みと扱わない。

## 自動CI

`ci.yml` がPR/main・develop push/手動実行の入口。`oss-profile.yml` を共通手順として呼ぶ。
MSRV1.92.0、x86_64 Linux musl、stable Ubuntu/macOS/Windowsを定義する。通常・最小構成、JPチェックサム、独立LSPを検査する。

- 全workspace書式、対応crate全targetのClippy（警告をエラー扱い）とテスト。
- 既存verifyの構造・ハッシュ・anchor・完全性・ZIP上限と29件のmain回帰。
- アーキテクチャ・公開範囲・依存ライセンス・文書リンクとPythonガード。
- 実release CLIのversion/help、veil-v1マスキング、0/1/2、不要な状態保存の検査。
- 正確なallowlistでZIPを作り、展開後の起動・checksum・原文/ソース・相対リンクを検査。
- macOSの非標準動的依存、Linux静的ELF、Windows通常/遅延DLL importを検査。

自動artifactはログ・JSON検証記録・checksumだけ。バイナリ/ZIPは自動公開しない。現在のhosted CI結果は未取得。
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

0.21.0-rc.1のLinux x86_64 musl静的候補はUbuntu 24.04.5 LTS / Linux 7.0.0-38-genericの実機でネイティブビルド・実行・同梱検査を確認。同じmusl targetの通常320件・最小319件・JP321件・LSP25件は失敗/ignored 0件。release LSPのstdio実起動・診断更新・修正候補・正常終了も確認。
Windowsネイティブ/MSVC、現行RCのARM64/他ディストリビューション/旧カーネル、IDE GUI連携、LSPバイナリ配布のライセンスpacket、現在のhosted CI、署名/公証/SmartScreenは未検証。
[Linuxガイド](../cli/linux-distribution.md)・[Windowsガイド](../cli/windows-distribution.md)の旧候補の結果を、現在のソースに流用しない。
公開するOS/CPU・番号・署名の扱いを決め、同じ承認済みソースで最終ZIPを生成・検証する。旧tagへ今回の変更を含めた扱いにしない。
crates.io公開、旧UI/サービス全体の動作確認、性能の優位性、サイト・テレメトリ・DBはこの準備の合格条件へ追加しない。
今回の実行結果・main向け差分・保全hashは別の準備レポートへ記録する。公開や元ブランチへの適用の承認を代替しない。
