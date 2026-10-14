# Local OSS Architecture

Status: `active`
Updated: 2026-10-14

## 責務

```text
veil-cli → veil-core → veil-config
     └──────────────→ veil-config
明示lsp操作 → 独立veil-lsp（stdio、有効設定を転送）
```

既存のcrate分割を維持し、通常default-membersはconfig/core/cli。通常CLIは外部API・ライセンス認証・UI/サーバー・テレメトリへ依存しない。旧メンバーは保全する。
mainにあるpreset・対話スキャン・テンプレート昇格・スキャナーAPI・ルール読込みエラーと設定の継承を維持する。
既存Evidence Pack v1のverifyは通常・最小構成へ含め、構造・ハッシュ・外部anchor・baseline・上限の検証を保全する。新規監査・台帳・署名v2・PatchGate/VEX機能と未使用breakerはこの差分から除外する。

## ガードと振る舞い

`python3 scripts/check_architecture.py` はdefault-members、逆依存、通常/build推移的グラフ、既知の通信API、公開範囲を確認する。代表的APIと定義した境界の回帰検査であり、任意のコードの無通信を形式的に証明しない。

- 旧remote設定は明示的な移行エラー。URLを診断へ出さない。
- GitのHTTPS/SSHを無効化し、vendored libgit2を指定。修正版と実リンクを検査する。
- JSONは既定で生の検知値・周辺行を隠す。CLIのunsafe/plainは明示的に生データを許可する。設定ファイルのplainは終了2で拒否する。
- 部分マスクは8文字以下を全て隠し、長い値は末尾4文字だけ表示する。
- baselineは4つの不完全フラグを保存前に拒否し、新規作成も既存変更も行わない。
- CI生成は取得元を明示し、結果をスキャン対象外のrunner一時領域へ保存する。
- `update` はローカル署名・TTL検証。既存ルールディレクトリを保全する。
- LSPは別ソース導入が必要。CLI配布ZIPには同梱せず、通常scanから起動しない。

## 設定・検査・書換えの境界

`scan`、`filter`、`mask`、`git scan` は同じCLIの有効設定ローダーと `try_get_all_rules` を使う。明示 `--config` は存在する通常ファイルを要求し、読込み・構文・ルールエラーを終了2へ伝播する。未指定の `veil.toml` がない場合は既定設定を維持する。従来の自動探索によるuser/org設定の警告付きフォールバックは維持し、明示環境変数による設定は厳格に扱う。

`scan_file` / `scan_data` の公開戻り値は互換性のため維持する。BINARY_FILE・MAX_FILE_SIZE・READ_ERRORのsentinel解釈はcoreの `FileScanStatus` に集約し、通常ファイルとGitで読取り異常を検知扱いに混ぜない。検出対象はコマンドごとに異なる。`filter` は標準入力の行、`git scan` は履歴の追加行、`mask` は明示した通常ファイルを扱う。

`mask` は全入力を事前確認し、同じディレクトリの一時ファイルから各入力を置換する。行末・末尾改行・権限を保持し、直前に入力バイト列を再確認する。既存backupとsymlinkは拒否する。複数ファイルの一括トランザクション、他プロセスへの排他、クラッシュ時のディレクトリエントリ永続化保証は提供しない。

## 公開準備

[リリース条件](../ci/oss-cli-release.md)と[Linux](../cli/linux-distribution.md)・[Windows](../cli/windows-distribution.md)の検証範囲を参照する。
統合前のOS候補の成功を現在のソースへ流用しない。Rust 1.92.0のUbuntu/macOS native・Linux x86_64 muslはhosted CIで検証する。Windowsネイティブ/MSVC、Developer ID署名・公証、性能優位性は未検証。
0.21.0-rc.1は公開済みプレリリース。公開済みZIPには後続のmain修正が入っておらず、ソースのCI成功で配布物の更新を意味しない。制限は[利用ガイドライン](../cli/usage-guidelines.md#公開rcと修正済みソースの区別)を参照する。公開tag v0.20.0と旧manifest0.17.0は過去の履歴として保全する。
局所のレビュー履歴は配布文書へ累積せず、準備レポートで保全する。旧ロードマップのタスクを新しい通常CLIの完成条件として自動復活させない。

0.21.0-rc.2は公開済みプレリリース。配布元 `449a1e2e` とPR #158マージ後main `b1d3234` のtree一致・main CI成功を確認した。公開後の文書更新で既存ZIPは差し替えない。
