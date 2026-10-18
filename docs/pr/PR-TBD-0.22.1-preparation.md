# 0.22.1準備 — 実開発の誤検出2件を含むパッチ版

## SOT
- Status: Draft
- Final goal: 実開発で安全に点検し、継続利用できること。
- Milestone: 発見した誤検出2件の修正を含む0.22.1の準備候補を検証する。
- Task: 版番号・CHANGELOG・関連文書、両OS検証、Draft PRのCI成功まで。

## What
- PR #172マージ後のmain `481406f` を基準に、CLI/core/config/LSP・workspace・Cargo.lock・Nixを0.22.1へ揃える。
- 外部依存を変更せず、Cargo.lockの固定ハッシュを第三者manifestへ反映する。
- 誤検出修正・通常の個人情報検出・IPv4維持をCHANGELOGへ記載する。
- 公開済み0.22.0の取得手順と未公開0.22.1候補を区別し、古い公開状態を修正する。
- 既存ソースの検出・出力・コマンド・導入処理は変更しない。

## Verification
- Cargo.lockの差分はローカル4パッケージの版番号だけで、外部依存レコードが不変であることを確認。
- macOS ARM64・Linux x86_64 muslでRust1.92.0による検査、CLI版表示・日英help・誤検出修正の回帰を検証する。
- CLI配布整合・第三者表記・文書分類・アーキテクチャ・SOTを確認する。
- head更新後のPR CI全件成功を完了条件とし、結果をPR本文へ記録する。

## Evidence
- 前提のmain CI: https://github.com/mt4110/veil-rs/actions/runs/39082115209
- 上記は準備前の基準であり、今回の変更のCI結果とは区別する。
- コード・検出値などの実利用データは記録しない。回帰は架空データのみ。

## Non-goals
- マージ・署名タグpush・Release作成／更新／公開。
- 公開済み0.22.0の成果物や既存変更、Crossportの通常インデックスの変更。
- 新機能や外部依存更新、検出基準の一般的な緩和。

## Rollback
- この準備PRを単位としてrevertする。公開済み成果物は不変。
