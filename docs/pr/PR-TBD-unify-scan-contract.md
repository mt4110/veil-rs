---
status: Draft
created_at: 2026-10-14
branch: feat/unify-scan-contract
title: Unify scanning configuration and safe file masking
---

# SOT

Scope: 設定・ルール読込み、検査不完全の扱い、maskの改行保持と安全な書換えを揃える。
基準main: `79cf4d92fb5181f683e310c50d16d5f1065efd76`。
元の未コミット変更は保全。マージ・公開・Release更新、他製品の操作は含めない。

# What

- 明示設定の欠損/非通常ファイルを既定設定へ戻さず終了2。未指定の設定欠損と既存自動探索のsoft fallbackは維持する。
- maskとGit履歴もCLIの有効設定とカスタム/ローカルRulePackを使う。Gitへ全体のconfig指定を渡す。
- coreのFileScanStatusへsentinel解釈を集約。通常scanのGit入力にも件数上限を適用し、baselineを保全する。
- 独立Git履歴は追加行の検出を維持し、変更先blob全体のUTF-8/サイズ、範囲内件数/検知上限を確認。不完全なら終了2。
- maskは明示通常ファイルのみ。全入力事前確認後に各入力をatomic置換。LF/CRLF・末尾改行・権限を保持し、変更された入力を拒否する。
- backupは元バイト列を新規保存し既存ファイル/symlinkを上書きしない。symlink/ディレクトリ/読取り専用の書換え対象・重複入力・backup衝突・空/パス区切りsuffixを拒否する。

全コマンドの検出対象や出力形式の統合、公開APIの破壊的変更、新しい抽象層、バージョン更新はしない。
複数ファイルの一括トランザクション/ロールバック、他プロセスへの排他、クラッシュ時のディレクトリエントリ永続化は保証しない。

# Verification

- scan/filter/mask/gitの明示設定欠損、設定継承・カスタム/RulePackの適用、壊れたRulePackのエラー伝播。
- LF/CRLF混在・末尾改行有無・複数秘密値・権限保持・backupバイト列一致。
- maskの不正UTF-8/サイズ/件数/検知上限で全入力とbackupを保持。
- ignore対象を件数予算から除外し、上限1件でも非ignoreファイルのdry-run/実書換え・backup作成が完了、ignore入力は保持。
- 既存backup・壊れたsymlink・入力alias/重複・衝突・読取り専用/ignore指定の安全境界。
- Git履歴の不正UTF-8/サイズ/件数/検知上限、binary除外、通常scan Git入力の件数上限とbaseline保全。
- 差し替わった入力snapshotの拒否を単体検査。
- Rust 1.92.0の通常/最小構成のconfig/core/cli全テストとClippy、両OS同一head CI、既存出力秘匿・配布・初回/ステージ/次回の回帰を確認。

# Evidence

修正前のmainと同一treeの実CLI・架空入力で設定/ルール/改行の動作差を再現。
macOS ARM64 / Rust 1.92.0の通常・最小構成でconfig/core/cli全テストとClippy all-targets成功。新しい実CLI回帰11件とsnapshot保全の単体検査を確認。Pythonガード98件、architecture、release metadata、docs taxonomy、fmt、diff check成功。同一headのLinux/macOS CI結果はPR本文へ記録する。
元作業ツリーstatus SHA256: `f29ac043b10d7fd2b9409c6f4e3f4b1748fed5b90887a116a4197109d775789b`。
元作業ツリーdiff HEAD SHA256: `62f4e299baffb30e0b0eaeda0b9c834ada48948f881a688055831d557141cb9e`。
公開済み0.21.0-rc.1 ZIPは未変更。架空入力の回帰と第三者利用・検出精度の保証は区別する。
