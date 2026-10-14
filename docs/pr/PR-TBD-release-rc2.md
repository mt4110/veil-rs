---
status: Draft
created_at: 2026-10-14
branch: feat/release-0.21.0-rc.2
title: Prepare 0.21.0-rc.2 CLI archives
---

# SOT

main `5ae098face5f671efc106a2848f2eaf7d8ae3f79` の修正を含むrc.2のLinux x86_64 musl・macOS ARM64 ZIPとSHA256SUMSを準備し、GitHub Releaseをドラフトで作成する。
既存の未コミット変更は保持。Release公開・署名タグpush・mainへのマージ・他製品の操作は含めない。

# What

対応4クレート/workspace依存/lock/Nixの版番号をrc.2へ統一し、文書にドラフト候補と公開済みrc.1の区別を記載。
配布元の最終コミット/ソース指紋を固定し、検証済みの2 ZIPと外側のSHA256SUMSのみをドラフトへ添付する。
macOSはad-hoc署名の試用版。Developer ID、公証、ダウンロード後Gatekeeperの保証はない。

# Verification

Rust 1.92.0、両OSの通常/最小構成・互換性CI、展開ZIPの実行・秘匿・入力/設定/書換え回帰、初回/ステージ/次回利用、同梱ライセンス/第三者表記、Linux静的リンクとmacOS署名整合を確認。
Linux実機ビルドは1ジョブ・CPU/メモリ制限・低い実行優先度で行う。
署名タグはローカルで署名検証し、push/公開は行わない。

# Evidence

結果は外部の新規候補保存先、PR本文、Releaseドラフト本文に記録する。未確認を合格扱いにしない。
元作業ツリーstatus SHA256: `f29ac043b10d7fd2b9409c6f4e3f4b1748fed5b90887a116a4197109d775789b`。
元作業ツリーdiff HEAD SHA256: `62f4e299baffb30e0b0eaeda0b9c834ada48948f881a688055831d557141cb9e`。
