# SOT

0.21.0正式版の版番号・変更記録・関連文書を整える。公開済みrc.4と既存の未コミット変更を保持し、マージ・タグpush・公開は行わない。

## What

workspace/CLI/core/config/LSP/Cargo.lock/Nixの版番号を0.21.0へ揃え、lock変更に対応するlicense inventory hashだけを更新する。外部依存と製品コードを変更しない。CHANGELOGは正式版未公開と明記し、旧0.22.0見出しを開発記録として保全する。READMEとチュートリアルは候補と公開rc.4の取得を区別し、古いmain参照とCIの仮タグを整理する。

## Verification

書式・版番号・依存/ライセンス・文書分類・リンク・Pythonガードを確認する。Draft PRの同一headでUbuntu/macOS native、Linux x86_64 musl、MSRV、guardとSOTを検証する。CIの展開ZIPとステージ済み導線を含む検査を確認し、最終配布用の署名済みZIP生成・公開の証拠とは区別する。

## Evidence

基準点はPR #166マージ後main 1db42b7（CI 38736368362成功）。今回の固定headと実行結果はPR本文に記録する。実作業での継続利用とmacOSのDeveloper ID・公証・Gatekeeperは未確認のまま保持する。
