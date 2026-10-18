# 0.22.1公開後の案内を同期

## SOT
- Status: Draft
- Goal: 公開済み0.22.1を取得し、初回・commit前・次回の点検へ迷わず進める。

## What
- README・日英チュートリアル・導入/PATH案内・CLI目次・スキャン仕様・Linux配布・Release記録・CHANGELOGの公開状態を同期。
- ダウンロードURL・ZIP名・版表示・署名タグを0.22.1へ揃える。
- 配布元のCI証拠と文書変更headのCIを区別する。macOSのad-hoc制限と既存ZIP不変を保持する。

## Verification
- 文書分類・CLI配布/相対リンク・アーキテクチャ・SOT・差分空白を検査。
- headに対するCI全7件成功を完了条件とする。
- 公開済みRelease/添付、既存変更、Crossportの通常インデックスが作業前と一致することを確認。

## Non-goals
- 製品コード・依存・版番号・公開済みZIPの変更、マージ、X投稿。

## Rollback
- この文書PRを単位としてrevertする。
