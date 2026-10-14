---
status: Draft
created_at: 2026-10-14
branch: fix/text-scan-boundaries
title: Harden five text scan input and output boundaries
---

# SOT

Scope: 対話プレビューの秘匿、Git入力の不正UTF-8拒否、inline ignoreの完全一致、TOML構文診断の本文秘匿、maskの設定エラー拒否。
基準main: `8f717c5dc56d415457dde662559c094378d8cdd5`。
既存未コミット変更は元チェックアウトで保全。マージ・公開・Release更新・他製品の操作は含めない。

# What

- 変更前プレビューは生の行から一致値1件だけを置換せず、全文を非表示にする。変更後は既存のmasked snippetを使う。
- NULのないGit blobのUTF-8変換失敗はREAD_ERROR。staged/commit/historyの完全性フラグを立て、終了2・baseline保存拒否を維持する。
- inline ignoreは引用符外の `#`/`//` コメント先頭の1トークン。全行指定か完全一致IDのみを認め、別ID・接尾辞・引用文字列を抑制しない。全言語構文の解析ではない。
- TOML構文・デシリアライズエラーは元のDisplayを診断へ流さず、行・列だけを伝える。パス等のメタデータ匿名化は含めない。
- maskは設定の読込み・検証失敗を伝播し、入力・バックアップへの書込み前に拒否する。

UTF-16復号、XML文字参照展開、画像/動画/OCR、隠しファイル/ignoreの全対象化、baseline照合方式、対話操作の終了条件、版番号は変更しない。

# Verification

実CLI回帰:
- 同一行の異なる2秘密値と反復一致を対話レビューし、quit前のプレビューとstderrに値が残らず入力を保持。
- 有効な全行/ID指定を保全し、別ID・接尾辞・single/double/backtick/escaped quote中の指定を検知する。
- 不正UTF-8のstaged/commit/historyで終了2、初回baseline非作成・既存baselineバイト列保持、index保持。
- 日本語を含む壊れたTOMLでscan/mask dry-run/実操作を終了2とし、秘密値・構文エラー行を表示せず、入力とバックアップを保全。

Rust 1.92.0のmacOS通常・最小構成、両OSの同一head CI、既存出力秘匿・完全性・配布・初回/ステージ/次回利用の検査を行う。

# Evidence

修正前はmainと同一treeのCLI・架空データで5件を再現済み。
macOS ARM64 / Rust 1.92.0の通常・最小構成でconfig/core/cli全テストとClippy all-targets成功。新しい実CLI回帰4件は各構成で成功。Pythonガード98件、architecture、release metadata、docs taxonomy、fmt、diff check成功。
Linux native/musl・macOSの同一head CIの結果はPR本文へ記録する。
元作業ツリーstatus SHA256: `f29ac043b10d7fd2b9409c6f4e3f4b1748fed5b90887a116a4197109d775789b`。
元作業ツリーdiff HEAD SHA256: `62f4e299baffb30e0b0eaeda0b9c834ada48948f881a688055831d557141cb9e`。
公開済み0.21.0-rc.1 ZIPは未変更。所有者の架空入力検証と第三者利用・検出精度の保証は区別する。
