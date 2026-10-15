---
status: Draft
created_at: 2026-10-15
branch: feat/rc3-release-prep
title: Prepare 0.21.0-rc.3 CLI release draft
---

# SOT

main e433582の未指定IPv4修正を含むrc.3のZIP2種とSHA256SUMSを準備・検証し、Releaseドラフトへ添付する。公開・X投稿・公開済みrc.2変更はしない。

# What

config/core/cli/LSPとworkspace・lock・Nixの版番号をrc.3に統一。既存梱包スクリプトでLinux x86_64 muslとmacOS ARM64を同一コミットから生成する。macOSはad-hocのみでDeveloper ID署名・公証なし。

# Verification

同一head CI、ネイティブでの展開ZIP/第三者表記/同梱checksum・21/12/10ケースとステージ済み変更の導線を検証する。未指定IPv4除外と非ゼロIPv4検出も最終バイナリで確認する。Linux CPU/メモリを制限する。元作業ツリー・公開済みrc.2のhashを照合する。

# Evidence

最終結果・コミット・ハッシュ・未検証範囲はローカル準備レポートとReleaseドラフト本文へ記録する。署名タグのpush・マージ・公開はしない。
