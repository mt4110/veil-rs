# Local OSS Architecture

Status: `active`
Updated: 2026-10-13

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
- JSONは既定で生の検知値・周辺行を隠す。unsafe/plainは明示的に生データを許可する。
- 部分マスクは8文字以下を全て隠し、長い値は末尾4文字だけ表示する。
- baselineは4つの不完全フラグを保存前に拒否し、新規作成も既存変更も行わない。
- CI生成は取得元を明示し、結果をスキャン対象外のrunner一時領域へ保存する。
- `update` はローカル署名・TTL検証。既存ルールディレクトリを保全する。
- LSPは別ソース導入が必要。CLI配布ZIPには同梱せず、通常scanから起動しない。

## 公開準備

[リリース条件](../ci/oss-cli-release.md)と[Linux](../cli/linux-distribution.md)・[Windows](../cli/windows-distribution.md)の検証範囲を参照する。
統合前のOS候補の成功を現在のソースへ流用しない。Windowsネイティブ/MSVC、現行hosted CI、署名・公証、性能優位性は未検証。
隔離候補の番号は0.21.0-rc.1。公開tag v0.20.0と旧manifest0.17.0は過去の履歴として保全する。原文ライセンス・依存lock・配布対象・実際の公開ソースの検証を確定する。
局所のレビュー履歴は配布文書へ累積せず、準備レポートで保全する。旧ロードマップのタスクを新しい通常CLIの完成条件として自動復活させない。
