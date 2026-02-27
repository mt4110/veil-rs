# veil-rs 🛡️
English README is available [here](README_EN.md).

**veil-rs** は、開発者および企業セキュリティチーム向けに設計された、ローカルファーストかつ高性能なシークレットスキャンツールです。コードベースに含まれる機密情報（APIキー、個人情報、クレデンシャル等）を驚異的な速度で検出します。

## 特徴
- **🚀 爆速スキャン**: Rust製で並列処理と正規表現最適化を駆使し、大規模リポジトリも瞬時にスキャン。
- **🇯🇵 日本の個人情報に対応**: マイナンバー、運転免許証番号、住所、電話番号などの日本固有フォーマットを高精度に検出。
- **👮 商用グレード機能**: TOMLによるカスタムルール定義、インライン無視機能 (`# veil:ignore`)、商用レベルのレポート出力。
- **📊 レポート機能**: 機械処理可能な JSON 出力に加え、提出用としてそのまま使える HTML ダッシュボード (`--format html`) を生成可能。
- **🛡️ 堅牢な制御**: `--fail-on-score` によるCI合否判定、`WARN` レベルの柔軟な運用が可能。
- **⚡ Staged Scan**: `--staged` オプションでコミット予定のファイルだけを高速スキャン。`pre-commit` に最適。


### 🛡️ データプライバシーとローカルファースト保証
Veil は B2B 環境での運用を前提に設計されており、厳格な「ローカルファースト」アーキテクチャに準拠しています。
- **外部通信ゼロ**: コード、スキャン結果、利用統計などのデータを外部サーバーへ送信することは一切ありません。
- **セキュアなローカル運用**: Veil Pro ダッシュボードは完全に `localhost` (127.0.0.1) で稼働し、外部アセット（CDN等）の依存を排除。トークンの持ち出しを防ぐ厳格な CSP（Content-Security-Policy）を強制します。
- **完全隔離**: スキャン、ベースラインの生成、レポート出力機能はすべて、ローカル環境またはCIランナー内のみで完結します。
- **📦 バイナリ/巨大ファイル対策**: バイナリファイルや1MB超の巨大ファイルは自動でスキップし、CIの詰まりや文字化けを防止。
- **🔧 完全設定可能 & 階層化**: `veil.toml` に加え、組織ごとの共通設定 (`VEIL_ORG_CONFIG`) を読み込む階層化ポリシー管理に対応。
- **💉 Stop the Bleeding (Baseline)**: 既存の技術的負債をスナップショット化し、"新規の漏洩" だけを確実に止める [Baseline Scanning](docs/baseline/usage.md) を標準搭載。

## Canonical Rules: RulePack (Source of Truth)

Veil’s rules are canonically defined as **RulePacks**.

A **RulePack** is a directory containing:

* `00_manifest.toml` (deterministic load order)
* one or more `.toml` files with `[[rules]]`

### Built-in (embedded) packs

Veil ships with embedded packs under:

* `crates/veil/rules/default/` (default rules)
* `crates/veil/rules/log/` (log scrubbing pack: OBS/SECRET/PII)

### Using a pack in your repo (`rules_dir`)

Point `core.rules_dir` to a RulePack directory:

```toml
[core]
rules_dir = "rules/log"
```

### Batteries-included log scrubbing

Generate a repo-local Log RulePack template:

```bash
veil init --profile Logs
```

## 📦 Install
### Cargo (Recommended)
```bash
cargo install --locked --git https://github.com/mt4110/veil-rs.git --tag v1.0.0 veil-cli
```
*(Requires Rust 1.82.0+)*

## ⏱️ 60-Second Quickstart

```bash
# 1. Initialize your project's configuration
veil init --wizard

# 2. Validate your rules
veil config check

# 3. Run your first scan
veil scan .
```

## 🖥️ Veil Pro ダッシュボード・クイックスタート
**Veil Pro ダッシュボード** は、日々の誤検知トリアージ、ノイズ管理、および監査証跡の作成を行う「ローカル向けコマンドセンター」です。B2Bグレードのセキュリティ（完全オフライン、ゼロ・テレメトリ）を備えています。

1. **ダッシュボードを起動**:
```bash
cargo run -p veil-pro
```
2. **安全にアクセス**: サーバーは `127.0.0.1` 以外にはバインドしません。`stderr` に出力された URL をブラウザで開いてください（例: `http://127.0.0.1:3000/#token=xxxxxxxx`）。`#token` のフラグメント認証により、サーバーログや履歴にクレデンシャルが漏洩するのを未然に防ぎます。
3. **外部公開厳禁**: リバースプロキシなどを用いてインターネットや 0.0.0.0 に公開しないでください。
4. **監査証跡の出力**: UI上の「Export Evidence Pack」ボタンを使うと、監査提出用に `report.html`, `report.json`, 実行メタデータ (`run_meta`) などを一括でZIP化してダウンロードできます。

### 🕵️ 第三者機関による証拠検証 (Golden Path)
外部通信ゼロ・完全オフラインで証拠の改ざんや漏洩がないかを検証する `verify` コマンドを提供しています。

1. **証拠の生成**: ダッシュボードから `evidence.zip` をエクスポートします。
2. **信頼のアンカーを記録**: ZIPに含まれる `run_meta.json` を抽出し、**生バイト列（raw bytes）のSHA256ハッシュ** を計算します。この値をチケットシステム等に記録してください。
   ```bash
   unzip -p evidence.zip run_meta.json | shasum -a 256
   # e.g. e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
   ```
3. **整合性の検証**: 監査担当者は提供された証拠パックと記録されたハッシュを突き合わせ、第三者検証を実行します。
   ```bash
   veil verify evidence.zip \
     --expect-run-meta-sha256 e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855 \
     --require-complete
   ```
* `Exit 0`: ZIP構造・ハッシュの一貫性・安全性が確認され、トークン漏洩がない証明。
* `Exit 1`: 証拠は正当だが、ポリシーに反している（`--require-complete` 指定時に不完全なスキャンだった等）。
* `Exit 2`: アーティファクトの改ざん、ZipBomb/ZipSlip攻撃の可能性、または明示的なトークン漏洩（`#token=`など）を検知。

## 🤖 CI Integration (GitHub Actions)

Fail the CI pipeline if the scan score exceeds a threshold (e.g., 50).

```yaml
- name: Veil Security Scan
  run: |
    # 1. Save HTML report as an artifact
    veil scan . --format html > veil-report.html

    # 2. Fail CI if findings exceed score 50
    veil scan . --fail-on-score 50
```

## 🩹 Stop the Bleeding (Baseline)

When introducing Veil to an existing codebase with legacy secrets, map them to a baseline so CI only fails on **new** secrets.

```bash
# Generate the baseline snapshot of current findings
veil scan . --format json > .veil-baseline.json

# Tell Veil to ignore these existing findings in future scans
export VEIL_BASELINE_FILE=.veil-baseline.json

# Scans will now only report NEW violations
veil scan .
```

## 🚦 Exit Codes
Veil uses strict exit codes to ensure robust CI automation.

| Code | Meaning        | Example                                                                                                |
| ---- | -------------- | ------------------------------------------------------------------------------------------------------ |
| `0`  | **Success**    | Scan completed with no fail-threshold violations.                                                      |
| `1`  | **Violation**  | Scan completed but findings exceeded `--fail-on-score`, `--fail-on-severity`, or `--fail-on-findings`. |
| `2`  | **Tool Error** | Scan aborted due to config error, reaching max limits, or baseline mismatch.                           |

## 🔇 Machine Output Purity (stdout / stderr)
For reliable automation, Veil strictly enforces output purity:

* **stdout**: Contains ONLY the requested machine format (e.g. valid JSON parsing via `--format json`).
* **stderr**: Contains ALL human-readable logs, warnings, progress bars, and diagnostics.

```bash
# This will NEVER corrupt the JSON file with random warnings!
veil scan . --format json > report.json
```

## Testing

veil-rs includes tests for secret detection rules (Slack, AWS, GitHub PATs, etc.).

To avoid GitHub Push Protection blocking pushes, we **never** hard-code real-looking secrets
as string literals. Instead, tests generate fake tokens at runtime via helper functions.

See [docs/TESTING_SECRETS.md](docs/TESTING_SECRETS.md) for the full “Safety Contract”
and guidelines on adding new secret tests.

### HTML Report (Triage with Browser)

Generate a rich, interactive HTML report with filtering and search capabilities. Perfect for manual review.

```bash
veil scan . --format html > report.html
# Then open report.html in your browser
```

## Configuration
## Integration Guide (JSON Output)

`veil-rs` produces a stable, machine-readable JSON output for integrations with CI/CD systems, dashboarding tools, and external verifiers (like `veri-rs`).

### Execution Example

```bash
veil scan . --format json --limit 1000 > veil-report.json
```

### Output Structure

The output is guaranteed to follow the [v1 Schema](docs/json-schema.md):

```json
{
  "schemaVersion": "veil-v1",
  "summary": {
    "findings_count": 5,
    "severity_counts": { "High": 2, "Medium": 3 },
    ...
  },
  "findings": [ ... ]
}
```

*   **`schemaVersion`**: Always check this field first (`veil-v1`).
    *   **Note**: We adhere to Semantic Versioning for the schema. `veil-v1` remains compatible for all v0.x releases. A structure-breaking change will increment this to `veil-v2`.
*   **`summary`**: Use this for high-level pass/fail decisions (e.g. `severity_counts.Critical > 0`).
*   **`findings`**: Full list of detected secrets.

---

## 商用運用ガイド

### 1. カスタムルールの追加 (Pure TOML)
Rust のコードを編集することなく、`veil.toml` に記述するだけで独自の検知ルールを追加できます。社内プロジェクトIDや特定のキーワードの検出に最適です。

```toml
[rules.internal_project_id]
enabled = true
description = "社内プロジェクトID (PROJ-XXXX)"
pattern = "PROJ-\\d{4}"
severity = "high"
score = 80
category = "internal"
tags = ["proprietary"]
```

### 2. インライン無視機能 (誤検知対応)
コード内のコメントで、特定の行の検知を無効化できます。

```rust
let fake_key = "AKIA1234567890"; // veil:ignore
let test_token = "ghp_xxxxxxxx"; // veil:ignore=github_personal_access_token
```

*   `// veil:ignore`: その行のすべての検知を無視します。
*   `// veil:ignore=rule_id`: 指定したルールIDの検知のみを無視します。


### 3. ポリシーの階層化 (Policy Layering)
全社共通のブラックリストや許容設定を一括管理できます。
環境変数 `VEIL_ORG_CONFIG` に共通設定ファイルのパスを指定すると、各プロジェクトの `veil.toml` とマージされます（プロジェクト設定が優先）。

```bash
export VEIL_ORG_CONFIG=/etc/veil/org_policy.toml
# org_policy.toml で "fail_on_score = 50" を設定しておけば、全プロジェクトで厳格なチェックを強制可能
```
> **Note:** 以前の環境変数 `VEIL_ORG_RULES` は **非推奨 (Deprecated)** となりました。今後は `VEIL_ORG_CONFIG` を使用してください。

### 3. CI/CD インテグレーション
GitHub Actions や GitLab CI ですぐに使えるテンプレートを `examples/ci/` に用意しています。

**GitHub Actions の例:**
```yaml
- name: Veil Security Scan
  run: |
    # HTMLレポートを生成（アーティファクト保存用）
    veil scan . --format html > report.html

    # スコア80以上の検出があればCI失敗
    veil scan . --fail-on-score 80
    
    # または、変更されたファイルだけをチェック (Pull Request時など)
    # veil scan --staged
```


### 4. Integrations

Detailed guides for integrating `veil-rs` into your workflow:

*   **[pre-commit Framework](docs/integrations/pre-commit.md)**: Drop-in support for `.pre-commit-config.yaml`.
*   **[Native Git Hooks](docs/integrations/git-hook.md)**: Simple shell script for `.git/hooks`.
*   **[GitHub Actions](docs/integrations/github-actions.md)**: CI integration template.

---

## ⚠️ Common Pitfalls & Troubleshooting (よくある落とし穴とトラブルシューティング)

**1. Exit Code 2: "Scan Incomplete" (Limit Reached)**
*   **What**: Veil stopped scanning because it hit the `max_file_count` or `max_findings` limit. This is a safety measure to prevent Out-Of-Memory errors or hanging CI jobs.
*   **How to fix**: 
    1.  **Reduce Scope**: Use `veil.toml` to specify `[core] ignore = ["tests/data", "dist"]` or scan specific folders (`veil scan src/`).
    2.  **Use Baseline**: If returning 10,000 existing secrets, use the Baseline feature to suppress them so you only process *new* findings.
    3.  **Increase Limit**: If you genuinely need to scan millions of files, increase `core.max_file_count` in `veil.toml`.

**2. Stdout vs. Stderr Purity (JSON broken)**
*   **What**: You ran `veil scan . --format json` in a script, but the JSON parser failed.
*   **Why**: You likely piped both `stdout` and `stderr` into your file (e.g., `> report.json 2>&1`). Veil strictly separates output: pure JSON/HTML goes to `stdout`, while human-readable logs, defaults skipped boundaries, and diagnostics go to `stderr`. 
*   **How to fix**: Only capture `stdout`! Use `veil scan . --format json > report.json`

**3. Org Config Overrides not applying**
*   **What**: You set `VEIL_ORG_CONFIG=/etc/veil/org_policy.toml`, but rules aren't changing.
*   **Why**: Local `veil.toml` settings always take precedence over the Org Config. If your local config specifies `fail_on_score = 100`, it will override the org policy.

**4. "I edited veil.ci.toml but my local scan ignores it!"**
*   **Why**: Veil automatically detects and loads `veil.ci.toml` *only* if the `CI=true` or `GITHUB_ACTIONS=true` environment variable is set. Otherwise, it prefers `veil.toml`.
*   **How to fix**: Explicitly pass the config file if you want to test CI rules locally: `veil scan --config veil.ci.toml .`

---

## ライセンス
Apache 2.0 または MIT ライセンスのデュアルライセンスです。

> **Note**: 現在は MIT/Apache-2.0 のOSSとして提供していますが、将来のバージョンでエンタープライズ向けの高度な機能については、異なるライセンス体系や有償アドオンとして提供する可能性があります（v0.x系はOSSのまま維持されます）。私たちは持続可能なOSS開発のために、最適なモデルを模索しています。
