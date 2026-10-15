---
status: Draft
created_at: 2026-10-15
branch: feat/staged-hook-safety
title: Respect Git-selected indexes and preserve pre-commit hooks
---

# SOT

M1の範囲で、Gitが選ぶステージ済み内容と生成フックの停止条件を揃える。既存の未コミット変更とrc.3ドラフトを保持。マージ・Release変更・公開・X投稿・他製品の操作はしない。

# What

- stagedはlibgit2のopen_from_envを使用。明示GIT_INDEX_FILEの空/欠損/非通常ファイルを拒否し、不正なindexのエラーを伝播する。baselineと両indexを保全する。
- 生成フックはfail-on-findings 1とredactを明示。検出による停止とCLI不在/実行エラーを区別する。
- Gitの有効hooks配置先とlinked worktreeを反映する。既存pre-commitの置換・退避・設定の変更は行わず、排他的な新規作成だけを許可する。
- 現行ソースと既存配布物の区別、既存フックへの手動統合とPATHの注意を文書へ記載する。

# Verification

修正前にフック6件とindex3件の回帰失敗を再現。修正後はindexの絶対/相対指定・通常indexとの逆転・異常系とbaseline保全、フックの正常/検出/エラー、既存ファイル/symlink/無効設定/別配置先の保全、linked worktree、実際のgit commit --onlyを検証する。macOSローカルとUbuntu/macOS/musl/MSRVの同一head CIを確認する。

# Evidence

macOSで回帰11件（通常/最小構成）、CLI/core/config全テスト、clippy -D warnings、Pythonガード関連検査が成功。固定コミットと両OSの同一head CI結果はPR本文へ記録する。rc.3は04745e7を対象にした既存ドラフトを変更しない。
