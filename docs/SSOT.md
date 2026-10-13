# Veil OSS SSOT

Status: `active`
Scope: ローカルCLIとスキャンエンジン
Updated: 2026-10-13

`veil scan` が通常の入口。ソースコードとログをローカルで検査し、マスキングした人間向け表示または `veil-v1` JSONを返す。初期化は任意。外部API・認証・テレメトリを前提にしない。

- `veil-config`: 設定型・読込み・継承・検証。
- `veil-core`: ルール・マスキング・並列スキャン・baseline・結果モデル・既存Evidence v1検証。
- `veil-cli`: 引数・設定レイヤー・Git読込み・出力・終了コード。
- `veil-lsp`: 明示LSPコマンドで起動する独立stdioサーバー。通常CLIのRust依存ではない。
- 旧UI/サーバー/Guardian: 既存ソースを保全。通常ビルド対象外。

不完全なスキャンのbaselineは保存せず終了2で既存を保全する。JSONは検知値と周辺行を既定で隠す。速度・精度の優位性を未測定のまま主張しない。
既存 `verify` とその入力契約は通常・最小構成で保全する。新規監査機能、台帳、署名v2・VEX/PatchGate契約は今回の公開範囲に含まない。

[アーキテクチャ](architecture/local-oss.md)・[リリース条件](ci/oss-cli-release.md)・[利用ガイド](cli/usage-guidelines.md)を参照。
公開・サイト・テレメトリ・DB・外部ディスク・旧ソースの物理削除は今回の準備に含まない。baselineは承認・修正完了や生値変更の識別を保証しない。
