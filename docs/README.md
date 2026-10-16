# Docs Portal

Use this page as the map. Current release conditions and candidate history live in [OSS CLI release status](ci/oss-cli-release.md); other pages explain product behavior, architecture, and historical designs. Historical or reference documents do not override current code, CI, or release status.

Status: `active`

通常利用は [README](../README.md)、製品の範囲は [SSOT](SSOT.md) を参照してください。公開済み0.21.0正式版・RCの履歴と検証範囲は[リリース状況](ci/oss-cli-release.md)で確認してください。
配布アーカイブでは、以下のガイドを含む選定済みの文書だけを同梱します。開発・検証コマンドは別途ソースを取得して実行します。

- [初回利用チュートリアル](cli/getting-started.md): 自分のファイルの初回スキャン、ステージ済み変更の点検、次回利用。架空入力の学習手順は任意。
- [ローカル導入・PATH案内](cli/install.md) / [English](cli/install.en.md): 開発中M1候補の導入。公開済み0.21.0のZIPは対象外。
- [利用ガイドライン](cli/usage-guidelines.md): 結果の読み方、安全な共有、スキップ・上限・書込み操作。
- [Linux配布ガイド](cli/linux-distribution.md): GNU/musl、CPU・libc・カーネルの条件と検証範囲。
- [Windows配布ガイド](cli/windows-distribution.md): GNU/MSVC、DLL・互換実行とネイティブ未検証の区別。
- [CLI](cli/README.md): スキャン、初期化、設定、ルール、診断。
- [OSS CLIのCI・リリース条件](ci/oss-cli-release.md)
- [アーキテクチャと残タスク](architecture/local-oss.md): 依存境界、ガード、互換機能、検証範囲。
- [開発ガイド](DEVELOPMENT.md) / [English](DEVELOPMENT_EN.md)
- [テスト用シークレットの規約](TESTING_SECRETS.md)
- [JSON契約](json-schema.md)
- [Baseline](baseline/usage.md)
- [脅威モデル](security/threat_model.md)
- [配布物の第三者ライセンスと同梱ソース](../THIRD_PARTY_NOTICES.md) / [NOTICE](../NOTICE)

`docs/architecture/**` と上記の製品・開発入口は active。
CLI・baseline・rules・integrations・runbook・security/threat_model は reference。
過去のops・PR・roadmap・設計・商用検討は、その時点の資料として読みます。通常CLIの仕様はSSOTと実装を優先します。roadmapや設計資料に残る未完了チェックリストを現行必須条件として復活させません。
細かい分類は `scripts/check_docs_taxonomy.py` のRULESを参照してください。

```bash
python3 scripts/check_docs_taxonomy.py
python3 scripts/check_architecture.py
```
