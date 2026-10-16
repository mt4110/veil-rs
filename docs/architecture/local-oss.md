# Local OSS Architecture

Status: `active`
Updated: 2026-10-15

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

`scan --staged` はlibgit2のGit環境変数対応の入口を使い、`GIT_INDEX_FILE` を含むインデックス選択を反映する。明示インデックスの空・欠損・非通常ファイル・読込み不正は終了2。生成フックは未抑制検出1件以上で停止し、実行エラーと区別する。導入はGitの有効hooks配置先へ排他的に新規作成し、既存pre-commitの置換・退避やcore.hooksPath変更を行わない。このM1修正は公開済みrc.2/旧rc.3候補（04745e7）には未反映。公開済みrc.3には反映済み。氏名ルール修正も含む0.21.0正式版は公開済み。

`mask` は全入力を事前確認し、同じディレクトリの一時ファイルから各入力を置換する。行末・末尾改行・権限を保持し、直前に入力バイト列を再確認する。既存backupとsymlinkは拒否する。複数ファイルの一括トランザクション、他プロセスへの排他、クラッシュ時のディレクトリエントリ永続化保証は提供しない。

## 配布状態と検証範囲

[現在のリリース条件・状態](../ci/oss-cli-release.md)を正本として、OS別の詳細は[Linux](../cli/linux-distribution.md)・[Windows](../cli/windows-distribution.md)の各記録を参照する。旧候補の成功を現行ソースや別OSへ流用しない。Windowsネイティブ/MSVC、Developer ID署名・公証、性能優位性は未検証。公開済み正式版・RCのタグ・ZIPは後続main変更で変わらず、main CI成功だけで配布物の更新を意味しない。
局所のレビュー履歴は配布文書へ累積せず、準備レポートで保全する。roadmapと設計資料の未完了タスクを現行通常CLIの完了条件として自動復活させない。
