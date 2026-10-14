---
status: Draft
created_at: 2026-10-14
branch: feat/ipv4-unspecified
title: Exclude unspecified IPv4 from PII findings
---

# SOT

未指定IPv4の誤検出だけを最小修正する。既存の未コミット変更を保持し、マージ・公開・X投稿・他製品の操作はしない。

# What

既存validator機構で4オクテットすべてがゼロの表記だけを除外する。ゼロ埋め表記も同様。通常・日本語・ログ向けIPv4ルールを揃える。正規表現、スコア、非ゼロIPv4、IPv6、認証情報ルールは変更しない。

# Verification

埋込みルールと同梱4種の回帰検査。未指定・ポート付き・ゼロ埋めの除外、各オクテットが非ゼロの境界、私用・公開・ループバック・文書用・ブロードキャストの検出維持、同一行の非ゼロIPv4検出を確認する。macOSローカルとUbuntu/macOS/muslの同一head CIを確認する。

# Evidence

修正前に回帰検査2件が失敗することを再現。macOSで新規回帰検査3件、veil-core全検査（既存の明示ignoredを除く）、clippy -D warnings、CLIビルドが成功。crossportの同じ2ファイルで公開rc.2の6件/exit 1が修正版0件/exit 0へ変化し、検査対象数2・スキップ0・読取りエラーなしを確認。Linuxを含む同一head CIの最終結果はPR本文に記録する。公開済みrc.2 ZIPは変更せず、この修正は未公開。crossportのコード・検出値は記録しない。
