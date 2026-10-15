# SOT

main 0149cd4の氏名ルール文脈判定修正を含む0.21.0-rc.4配布候補を固定する。既存の未コミット変更、公開済みrc.3を保持し、Releaseはドラフトまで。タグpush・公開・X投稿はしない。

## What

workspace/config/core/CLI/LSP/NixとCargo.lock内の自プロジェクト版番号をrc.4へ揃え、第三者表記manifestのlock hashを更新する。外部依存・製品コードはmainから変更しない。同梱ガイドのrc.3公開状態とrc.4候補の入口を整合させる。

## Verification

同一固定ソースからmacOS ARM64とLinux x86_64 muslのZIPをネイティブビルドし、展開後CLI、版表示、秘匿・異常系・RulePack・Git導線・氏名回帰、ライセンス/第三者表記/チェックサムを検査する。macOSはad-hoc署名のみ。Linuxは1ビルドジョブ・CPU/メモリ制限・既存キャッシュで検証する。

## Evidence

配布元SHA、両OSの結果、添付ハッシュ、ドラフト状態は準備記録とRelease本文へ記録する。未確認の別日実利用とDeveloper ID・公証・ダウンロード時Gatekeeper動作は成功扱いにしない。
