---
status: Draft
created_at: 2026-10-15
branch: feat/rc3-m1-candidate
title: Refresh rc.3 draft with M1 fixes
---

# SOT

main bded3aeのM1を含むrc.3候補を固定し、両OSの最終ZIPとSHA256SUMSを検証して既存Releaseドラフトを更新する。旧候補・元作業ツリー・公開済みrc.2を保持し、公開・X投稿・タグpush・mainマージはしない。

# What

workspace/config/core/cli/LSP/Nixとlockをrc.3に揃え、依存の内容は変えずlicense inventoryのlock hashを更新する。旧候補04745e7と新候補の文書を区別し、展開済みrc.3の利用手順を補足する。製品コードはbded3aeのまま。

# Verification

配布元の同一SHA CI、両OSのネイティブビルド・最終ZIP展開/ライセンス/チェックサム/既存21・12・11ケース/ステージ導線を確認する。最終バイナリでM1の別index・フック停止・既存hook保全・部分コミットも確認する。Linuxは既存キャッシュを利用しCPU/メモリを制限する。旧draftはダウンロードとmetadataを保全してから更新し、更新後の添付を再取得して照合する。

# Evidence

固定ソース・ビルド環境・検証結果・ハッシュ・Release状態はローカル準備レポートとドラフト本文へ記録する。未検証を成功扱いにしない。
