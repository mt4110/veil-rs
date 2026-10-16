//! Static help text. Identifiers, values and machine output remain unchanged.
use super::Language;
#[derive(Clone, Copy)]
pub struct Text {
    en: &'static str,
    ja: &'static str,
}
impl Text {
    pub fn get(self, lang: Language) -> &'static str {
        match lang {
            Language::En => self.en,
            Language::Ja => self.ja,
        }
    }
}
const fn t(en: &'static str, ja: &'static str) -> Text {
    Text { en, ja }
}
pub fn command(path: &str) -> Option<(Text, Text)> {
    Some(match path {
        "" => (t("Local secret scanning and masking", "ローカルでシークレットを検出・マスキング"), t("Examples:\nveil scan main.py --mask-mode redact --fail-on-findings 1\nveil scan --staged --mask-mode redact --fail-on-findings 1\n\nExit codes: 0 below thresholds, 1 findings threshold, 2 error/incomplete scan.\nIgnored files and empty stages are not proof of safety.\nHelp language does not translate reports or runtime errors.", "実行例:\nveil scan main.py --mask-mode redact --fail-on-findings 1\nveil scan --staged --mask-mode redact --fail-on-findings 1\n\n終了コード: 0は失敗条件未達、1は検出条件該当、2はエラー・不完全。無視対象や空ステージは安全の証明ではありません。言語指定はレポートや実行エラーを翻訳しません。")),
        "scan" => (t("Scan files and directories using local rules", "ローカルルールでファイルやディレクトリを点検"), t("Examples:\nveil scan main.py --mask-mode redact --fail-on-findings 1\nveil scan --staged --mask-mode redact --fail-on-findings 1\n\nNo initialization required. Scan does not rewrite source files. Restage fixes before scanning --staged. Exit 0/1/2 distinguishes thresholds from errors/incomplete input. Baseline writes reject incomplete input with exit 2 and preserve existing baselines. --unsafe/plain can expose secrets.", "実行例:\nveil scan main.py --mask-mode redact --fail-on-findings 1\nveil scan --staged --mask-mode redact --fail-on-findings 1\n\n初期化は不要。元のソースは変更しません。--stagedは修正後に再ステージが必要です。終了0/1/2で検出条件とエラー・不完全を区別します。不完全なbaseline保存は終了2で拒否し既存を保全します。--unsafe/plainは秘密を露出します。")),
        "filter" => (t("Filter STDIN and mask secrets (outputs to STDOUT)", "標準入力のシークレットを秘匿して標準出力へ出す"), t("Examples:\nprintf 'setting=true\\n' | veil filter\n\nReads stdin; does not scan a project or Git history. Masking is not complete anonymization.", "実行例:\nprintf 'setting=true\\n' | veil filter\n\n標準入力だけが対象で、プロジェクトやGit履歴全体は検査しません。マスキングは完全な匿名化ではありません。")),
        "init" => (t("Initialize a new configuration file", "設定ファイルを作成する"), t("Examples:\nveil init --non-interactive\n\nOptional: built-in rules work without initialization. Existing configuration is preserved. --wizard has its own language selector.", "実行例:\nveil init --non-interactive\n\n初回に必須ではありません。組み込みルールは初期化なしで使えます。既存設定は保全します。--wizardの言語選択は別機能です。")),
        "config" => (t("Configuration tools", "設定の確認と表示"), t("Examples:\nveil config check\n\nInspect effective settings without regenerating configuration.", "実行例:\nveil config check\n\n設定を再生成せず、有効な内容を確認します。")),
        "config check" => (t("Validate configuration and rules", "設定とルールを検証する"), t("Examples:\nveil config check\n\nAn error is not a clean scan. This command validates configuration; it does not scan all source files.", "実行例:\nveil config check\n\nエラーを検出なしとは扱いません。この操作は設定の検証であり、全ソースのスキャンではありません。")),
        "config dump" => (t("Dump configuration (preset/org/user/repo/effective)", "設定レイヤーを表示する"), t("Examples:\nveil config dump --layer effective --format json\n\nMay reveal local paths and settings. Review before sharing output.", "実行例:\nveil config dump --layer effective --format json\n\nローカルのパスや設定を含む場合があります。共有前に出力を確認してください。")),
        "pre-commit" => (t("Pre-commit hook tools", "commit前フックの導入"), t("Examples:\nveil pre-commit init\n\nManual staged scans remain available without a hook.", "実行例:\nveil pre-commit init\n\nフックを導入しなくても手動のステージ済み検査は利用できます。")),
        "pre-commit init" => (t("Install the pre-commit hook", "commit前フックを導入する"), t("Examples:\nveil pre-commit init\n\nRequires a Git repository and veil on PATH. Preserves existing hooks and hooksPath; refuses conflicts. The generated hook stops on findings or execution errors.", "実行例:\nveil pre-commit init\n\nGitリポジトリとPATH上のveilが必要です。既存フックとhooksPathを保全し、衝突は拒否します。生成フックは検出・実行エラーで停止します。")),
        "git" => (t("Git history and operations", "Git履歴の点検"), t("Examples:\nveil git scan HEAD\n\nFor the current index, use veil scan --staged.", "実行例:\nveil git scan HEAD\n\n現在のインデックスの点検にはveil scan --stagedを使います。")),
        "git scan" => (t("Scan git history", "Git履歴の追加行を点検する"), t("Examples:\nveil git scan HEAD\n\nRequires Git objects. This history path is distinct from filesystem scanning and scan --staged. Read/decoding/limit failures return 2.", "実行例:\nveil git scan HEAD\n\nGitオブジェクトが必要です。ファイル点検やscan --stagedとは対象が異なります。読込み・復号・上限のエラーは終了2です。")),
        "doctor" => (t("Show diagnostic definition and system info", "環境・設定・ローカルルールを診断する"), t("Examples:\nveil doctor\n\nDiagnostics do not prove that source files contain no secrets.", "実行例:\nveil doctor\n\n診断の成功は、ソースに秘密情報がないことを保証しません。")),
        "rules" => (t("Rules related commands", "ローカルルールの確認と管理"), t("Examples:\nveil rules list\n\nBuilt-in rules need no download or update.", "実行例:\nveil rules list\n\n組み込みルールの利用にダウンロードや更新は不要です。")),
        "rules list" => (t("List all effective local rules", "有効なローカルルールを一覧表示する"), t("Examples:\nveil rules list\n\nUses effective configuration and configured local RulePacks.", "実行例:\nveil rules list\n\n有効な設定と、設定済みのローカルRulePackを読みます。")),
        "rules explain" => (t("Show detailed information for a specific rule", "指定ルールの詳しい説明を表示する"), t("Examples:\nveil rules explain creds.aws.access_key_id\n\nRule IDs are language-independent. Rule descriptions are not fully translated.", "実行例:\nveil rules explain creds.aws.access_key_id\n\nルールIDは言語で変えません。ルール原文全体の翻訳は対象外です。")),
        "rules promote-templates" => (t("Promote selected inactive JP templates into an executable RulePack", "選択した未有効のJPテンプレートからRulePackを生成する"), t("Examples:\nveil rules promote-templates --templates-dir /path/to/templates --out-dir /path/to/new-pack --min-score 80 --dry-run\n\nReview selection with --dry-run. Writing requires an output directory; --force permits replacing generated files. The pack must be configured before scanning uses it.", "実行例:\nveil rules promote-templates --templates-dir /path/to/templates --out-dir /path/to/new-pack --min-score 80 --dry-run\n\n--dry-runで選択内容を確認してください。生成には保存先が必要です。--forceは生成物の置換を許可します。スキャンで使うにはRulePackの設定が必要です。")),
        "lsp" => (t("Run the Veil language server over stdio", "標準入出力でVeil言語サーバーを起動する"), t("Examples:\nveil lsp --preset fintech-jp\n\nRequires a separately installed veil-lsp on PATH.\nThe CLI ZIP does not include the server; normal scans do not start it.", "実行例:\nveil lsp --preset fintech-jp\n\nPATH上に別途導入したveil-lspが必要です。CLI ZIPにサーバー本体は含めず、通常スキャンでは起動しません。")),
        "update" => (t("Install a local signed RulePack (requires --local-pack)", "署名済みローカルRulePackを検証して配置する"), t("Examples:\nveil update --local-pack rules.zip --public-key publisher.hex\n\nRequires matching rules.sig and a trusted hex-encoded Ed25519 public key. Preserves existing .veil/rules. This command neither downloads nor automatically activates the pack; configure core.rules_dir separately.", "実行例:\nveil update --local-pack rules.zip --public-key publisher.hex\n\n対応するrules.sigと信頼するEd25519公開鍵（hex形式）が必要です。既存の.veil/rulesを保全します。ダウンロードや自動有効化は行わず、core.rules_dirは別途設定します。")),
        "verify" => (t("Verify an Evidence Pack v1 ZIP", "Evidence Pack v1のZIPを検証する"), t("Examples:\nveil verify evidence.zip --require-complete\n\nChecks existing v1 integrity and policy, not a new source scan. Exit 0 valid, 1 policy violation, 2 verification/input error.", "実行例:\nveil verify evidence.zip --require-complete\n\n既存v1の整合性とポリシーを確認し、新たなソーススキャンは行いません。終了0は有効、1はポリシー違反、2は検証・入力エラーです。")),
        "help" => (t("Print this message or the help of the given subcommand(s)", "指定したコマンドのhelpを表示する"), t("Examples:\nveil --lang ja help scan\n\nExamples and identifiers describe the existing CLI.", "実行例:\nveil --lang ja help scan\n\n例と識別子は既存CLIの契約に従います。")),
        _ => return None,
    })
}
pub fn argument(path: &str, id: &str) -> Option<Text> {
    let specific = match (path, id) {
        ("init", "wizard") => Some(t("Run interactive wizard (separate language selection)", "対話ウィザードを実行（言語選択は別機能）")),
        ("init", "non_interactive") => Some(t("Script mode: preserve an existing configuration and fail", "スクリプト用: 既存設定を保全して失敗する")),
        ("init", "ci") => Some(t("Generate CI for a provider (github)", "指定プロバイダーのCIを生成（github）")),
        ("init", "profile") => Some(t("Use a profile without the wizard (e.g. Logs)", "ウィザードなしでプロファイルを適用（例: Logs）")),
        ("init", "ci_source") => Some(t("CI source: checkout or a full 40-character Veil commit SHA", "CIの取得元: checkoutまたはVeilの完全な40文字コミットSHA")),
        ("init", "pin_tag") => Some(t("Explicit release tag for compatibility; not the current working tree", "互換用の明示リリースタグ。現在の作業ツリーを示しません")),
        ("config check", "config_path") => Some(t("Override configuration file path", "設定ファイルのパスを上書き指定する")),
        ("config dump", "layer") => Some(t("Layer: preset, org, user, repo, effective (default)", "表示レイヤー: preset, org, user, repo, effective（既定）")),
        ("config dump", "format") => Some(t("Output format: json (default) or toml", "出力形式: json（既定）またはtoml")),
        ("git scan", "range") => Some(t("Git range (default: HEAD; e.g. HEAD~5..HEAD)", "Git範囲（既定: HEAD、例: HEAD~5..HEAD）")),
        ("git scan", "pr") => Some(t("Scan against upstream or origin/main; conflicts with a range", "upstreamまたはorigin/mainとの差を検査。範囲指定とは併用不可")),
        ("rules explain", "rule_id") => Some(t("Rule ID (e.g. creds.aws.access_key_id)", "ルールID（例: creds.aws.access_key_id）")),
        ("scan", "paths") => Some(t("Paths to scan (default: current directory)", "点検するパス（既定: 現在のディレクトリ）")),
        ("scan", "format") => Some(t("Output format: text (default), json, html, table, md or markdown", "出力形式: text（既定）, json, html, table, md, markdown")),
        ("scan", "fail_score") => Some(t("Exit 1 if any score reaches this value (0-100; env: VEIL_FAIL_SCORE)", "いずれかのスコアが指定値以上なら終了1（0〜100、環境変数: VEIL_FAIL_SCORE）")),
        ("scan", "fail_on_severity") => Some(t("Exit 1 if any score reaches the severity floor: Low, Medium, High, Critical", "指定した重大度の最小スコア以上なら終了1（Low, Medium, High, Critical）")),
        ("scan", "fail_on_findings") => Some(t("Exit 1 if new, unsuppressed findings reach this count (>= 1)", "未抑制の新規検出が指定件数以上なら終了1（1以上）")),
        ("scan", "commit") => Some(t("Scan a specific Git commit SHA", "指定したGitコミットSHAを点検する")),
        ("scan", "since") => Some(t("Scan Git history since a time; not filesystem mode", "指定日時以降のGit履歴を点検。ファイル点検モードでは使いません")),
        ("scan", "staged") => Some(t("Scan index contents, including GIT_INDEX_FILE; restage after fixes", "GIT_INDEX_FILEを含むインデックスの内容を点検。修正後は再ステージが必要です")),
        ("scan", "interactive") => Some(t("Start interactive review; not usable for baseline writes", "対話レビューを開始。baseline保存とは併用できません")),
        ("scan", "progress") => Some(t("Show progress when output is a terminal", "端末への出力時に進行状況を表示する")),
        ("scan", "mask_mode") => Some(t("Masking mode: redact, partial, plain (plain exposes raw secrets)", "秘匿方式: redact, partial, plain（plainは生の秘密情報を表示）")),
        ("scan", "unsafe_output") => Some(t("Reveal raw secrets (alias for --mask-mode plain)", "生の秘密情報を表示（--mask-mode plainの別名）")),
        ("scan", "limit") => Some(t("Finding limit (0 = unlimited); truncation returns exit 2", "検出件数の上限（0は無制限）。上限による不完全検査は終了2")),
        ("scan", "baseline") => Some(t("Suppress findings matching this baseline; not a remediation approval", "一致した検出をbaselineで抑制。修正完了の承認ではありません")),
        ("scan", "write_baseline") => Some(t("Save baseline: exit 0 on success, 2 on error/incomplete scan; preserve an existing baseline on failure", "baseline保存: 成功は終了0、エラー・不完全は終了2。失敗時は既存baselineを保全")),
        ("rules promote-templates", "templates_dir") => Some(t("Template pack root with MANIFEST.csv and templates/", "MANIFEST.csvとtemplates/を含むテンプレートpackのルート")),
        ("rules promote-templates", "out_dir") => Some(t("Output RulePack directory", "出力RulePackのディレクトリ")),
        ("rules promote-templates", "category") => Some(t("Select categories; repeatable", "対象カテゴリ（繰り返し指定可）")),
        ("rules promote-templates", "variant") => Some(t("Select variants; repeatable", "対象のvariant（繰り返し指定可）")),
        ("rules promote-templates", "severity") => Some(t("Select severities; repeatable: Low, Medium, High, Critical", "対象の重大度（繰り返し指定可: Low, Medium, High, Critical）")),
        ("rules promote-templates", "min_score") => Some(t("Include scores >= this value", "この値以上のスコアを含める")),
        ("rules promote-templates", "max_score") => Some(t("Include scores <= this value", "この値以下のスコアを含める")),
        ("rules promote-templates", "pack_id") => Some(t("Generated pack ID (default: veil.jp.security.promoted)", "生成するpackのID（既定: veil.jp.security.promoted）")),
        ("rules promote-templates", "dry_run") => Some(t("Preview selection without writing", "書き込まずに選択内容を確認する")),
        ("rules promote-templates", "force") => Some(t("Allow replacement of generated manifests and template copies", "生成したmanifestとテンプレートのコピーの置換を許可する")),
        ("rules promote-templates", "allow_all") => Some(t("Allow selection of all templates with no filters", "フィルターなしで全テンプレートの選択を許可する")),
        ("update", "public_key") => Some(t("Trusted hex-encoded Ed25519 key for RulePack signature verification", "RulePack署名を検証する信頼済みEd25519公開鍵（hex形式）")),
        ("update", "local_pack") => Some(t("Local signed RulePack ZIP; downloads are disabled", "署名済みローカルRulePackのZIP。ダウンロードは行いません")),
        ("verify", "path") => Some(t("Evidence Pack v1 ZIP to verify", "検証するEvidence Pack v1のZIP")),
        ("verify", "format") => Some(t("Output json for machine-readable results; otherwise text (default)", "機械可読の結果はjson。それ以外はtext（既定）")),
        ("verify", "require_complete") => Some(t("Require the evidence pack to be complete", "Evidence Packの完全性を必須にする")),
        ("verify", "expect_run_meta_sha256") => Some(t("Expected external run_meta SHA-256 anchor", "外部run_metaの期待するSHA-256 anchor")),
        ("verify", "max_zip_bytes") => Some(t("Maximum ZIP bytes (default: 524288000)", "ZIPのバイト上限（既定: 524288000）")),
        ("verify", "max_entry_bytes") => Some(t("Maximum expanded bytes per entry (default: 209715200)", "展開後の1項目のバイト上限（既定: 209715200）")),
        ("verify", "max_total_bytes") => Some(t("Maximum expanded bytes in total (default: 1073741824)", "展開後の合計バイト上限（既定: 1073741824）")),
        ("verify", "max_files") => Some(t("Maximum file count (default: 64)", "ファイル件数の上限（既定: 64）")),
        _ => None,
    };
    specific.or_else(|| {
        Some(match id {
            "lang" => t(
                "Language for help (ja/en); does not translate scan output",
                "helpの言語（ja/en）。スキャン出力は翻訳しません",
            ),
            "config" => t(
                "Path to config file (default: ./veil.toml)",
                "設定ファイルのパス（既定: ./veil.toml）",
            ),
            "no_color" => t("Disable colored output", "色を付けずに表示する"),
            "quiet" => t("Suppress non-essential output", "必須でない出力を抑える"),
            "help" => t("Print help", "helpを表示する"),
            "version" => t("Print version", "版番号を表示する"),
            "command" => t("Command path to describe", "helpを表示するコマンドの並び"),
            "preset" => t(
                "Apply a built-in preset as the base config layer",
                "組み込みプリセットを設定の基礎レイヤーに使う",
            ),
            "severity" => t(
                "Minimum severity (Low, Medium, High, Critical)",
                "最小の重大度（Low, Medium, High, Critical）",
            ),
            "fail_on_findings" => t(
                "Exit 1 if the finding count reaches this threshold (>= 1)",
                "検出件数が指定値以上なら終了1（1以上）",
            ),
            _ => return None,
        })
    })
}
