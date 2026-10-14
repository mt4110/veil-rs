---
status: Draft
created_at: 2026-10-14
branch: fix/table-output-masking
title: Mask table matches unless raw output is explicitly enabled
---

# SOT

Scope: 公開CLIのtable出力の秘匿、出力形式の回帰検査、関連文書の整合。
既存未コミット変更は別チェックアウトで保全。マージ・Release更新・公開・他製品への変更は含めない。

# What

公開0.21.0-rc.1の `scan --format table --mask-mode redact` は検知した生値を表示する。
tableのMatch列を既定で `<REDACTED>` にし、JSONと同じ明示生出力判定を使う。
CLIの `--unsafe`、`--mask-mode plain` は生出力を許可する。設定plainはredact同時指定でも終了2で拒否する。
text/JSON/HTML/Markdown/md/tableの実CLI回帰を通常・最小構成と配布検査に追加する。
パス、未検出文字列の匿名化、baseline契約変更、版番号変更は含めない。

# Verification

- 公開macOS ARM64 ZIPで修正前のtable生値出力を架空値で再現済み。
- macOS ARM64 / Rust 1.92.0：通常・最小構成の出力形式回帰（各42条件）、両構成のClippy all-targets成功。
- Linux native/muslとmacOSの同一head CIを確認する。pi-nodeは負荷が高いため重いビルドを追加しない。
- 既存0/1/2・初回/ステージ/次回利用の回帰と既存配布ガードを維持する。

# Evidence

- 新しい配布ガードは旧公開macOS ZIPを `table, safe.toml, ()` の秘匿違反で拒否し、修正済みCLIでは成功。
- 既存CLIのbaseline 12件、JSON schema 1件、OSS profile 5件、初回/ステージ/次回利用17ケース成功。
- Pythonガード98件、architecture、release metadata、docs taxonomy、fmt、diff check成功。
- CIはDraft PRの同一headでLinux/macOSの通常・最小構成、release CLIと展開ZIPを検証する。結果リンクはPR本文へ記録する。
- 元作業ツリーのstatus SHA256は `f29ac043b10d7fd2b9409c6f4e3f4b1748fed5b90887a116a4197109d775789b`、diff HEAD SHA256は `62f4e299baffb30e0b0eaeda0b9c834ada48948f881a688055831d557141cb9e`。作業前後で照合する。
- 既存公開ZIPは変更していない。42条件は形式別の同じ入力の組合せで、固有不具合・利用者実績の件数ではない。
