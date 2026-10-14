---
status: Draft
created_at: 2026-10-14
branch: feat/docs-rc2-published
title: Update published rc.2 documentation and downloads
---

# SOT

READMEとチュートリアルを公開済み0.21.0-rc.2の記述・取得手順へ更新する。
既存の未コミット変更を保全。マージ・公開済みZIPの差し替え・他製品の操作はしない。

# What

- README日英のrc.2ドラフト表記を公開済みに変更し、Releaseと取得手順へリンクする。
- Ubuntuの取得URL・ZIP名・期待バージョン・保存先例をrc.2へ統一する。
- rc.1の検証は履歴と明示し、rc.1の不具合注意はrc.2への移行案内にする。
- リンク先の有効な設計/配布/利用/scan文書も、rc.2の公開状態だけ最小修正する。Linux配布ガイドのReleaseリンク・方式表・ZIP公開状態を揃え、rc.1の環境記録は履歴として保持する。
- macOSのad-hoc署名・Developer ID/公証なし・Gatekeeper未検証の制限を保全する。

# Verification

公開Release・添付名/ハッシュとの照合、Ubuntu取得ブロックのLinux実行、公開CLIの初回/ステージ/次回利用、文書リンク/分類/リリースmetadata、同一head CIを確認する。
コード・依存・CLI引数は変更せず、既存のガードと実行検査を使う。

# Evidence

Release v0.21.0-rc.2はdraft=false/prerelease=true、ZIP2種とSHA256SUMSを公開済み。
Linux実機でチュートリアルの取得ブロックを実行し、公開rc.2のダウンロード・チェックサム・展開・版表示と初回/ステージ/次回利用を確認済み。metadata/link・architecture・docs taxonomy・diff check成功。同一head CIの最終結果はPR本文へ記録する。公開ZIPは準備時点の文書を含み、今回の文書更新で再梱包しない。
元作業ツリーstatus SHA256: `f29ac043b10d7fd2b9409c6f4e3f4b1748fed5b90887a116a4197109d775789b`。
元作業ツリーdiff HEAD SHA256: `62f4e299baffb30e0b0eaeda0b9c834ada48948f881a688055831d557141cb9e`。
