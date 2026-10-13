# Command Line Interface

Status: `reference`
Primary scope: [SSOT](../SSOT.md)

まず [チュートリアル](getting-started.md)で架空の入力をスキャンし、[利用ガイドライン](usage-guidelines.md)で結果の扱いを確認してください。組み込みルールで実行でき、初期化は任意です。
配布先の条件は [Linux配布ガイド](linux-distribution.md) / [Windows配布ガイド](windows-distribution.md) を参照してください。
CIで新規検知1件以上を失敗にする場合は `--fail-on-findings 1`。baselineを指定するだけではこの条件は追加されません。

- [scan](scan.md): 人間向け / `veil-v1` JSON、Git、baseline、終了条件。
- [init](init.md): 任意の設定ファイル・CI設定生成。リモートルール設定は新規生成しません。
- `rules list` / `rules explain <id>`: ローカルの有効ルール。
- `config check` / `config dump`: 設定の検証・表示。
- `doctor`: 設定や環境の診断。
- `filter`: 標準入力をマスキング。
- `git scan`: Git履歴のローカル検査。
- `pre-commit init`: フックの導入。
- `update --local-pack <zip> --public-key <hex-file>`: 署名付きローカルRulePackの初回導入。既存 `.veil/rules` は保全。

`guardian` は通常CLIには含みません。[旧クレートの扱い](../architecture/local-oss.md) を参照してください。旧Guardianの操作資料は通常CLIの配布文書には含めません。
`verify <pack.zip>` は既存Evidence Pack v1の検査です。通常・最小構成で利用でき、`--require-complete`、外部anchor、件数条件、ZIP予算を指定できます。生tokenに一致した診断はその値を表示しません。パック単体のハッシュ一致は作成者の認証ではなく、由来を固定する場合は外部で保持したrun_metaのSHA-256を指定してください。通常scan JSONや配布CLI ZIPはEvidence Packではありません。
新規の台帳・署名v2・Detect/Prove/Decide・PatchGate/VEX機能はこの候補には含みません。

```bash
veil --help
veil scan --help
```


## 現行mainから維持した操作（2026-10-13）

`scan --preset fintech-jp` 等の組み込みpreset、`scan --interactive`、`rules promote-templates`、設定の継承・入力エラー検出を維持しています。`logs-jp` は `veil init --preset logs-jp` でローカルのlogルールを準備してから使用します。詳細な選択肢は各コマンドの `--help` を参照してください。
部分マスクは8文字以下を全て隠し、9文字以上は末尾4文字だけを表示します。先頭4文字は表示しません。

`veil lsp --preset fintech-jp` は独立した `veil-lsp` を標準入出力で起動します。CLI配布ZIPにはLSP本体を同梱しません。ソースから `cargo install --locked --path crates/veil-lsp` で導入し、PATHへ追加してください。設定・presetはCLIから子プロセスへ渡します。通常スキャンはLSPを起動せず、LSPのサーバー依存をリンクしません。
