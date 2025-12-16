# veil-rs 🛡️
English README is available [here](README_EN.md).

**veil-rs** は、開発者および企業セキュリティチーム向けに設計された、ローカルファーストかつ高性能なシークレットスキャンツールです。コードベースに含まれる機密情報（APIキー、個人情報、クレデンシャル等）を驚異的な速度で検出します。

## 特徴
- **🚀 爆速スキャン**: Rust製で並列処理と正規表現最適化を駆使し、大規模リポジトリも瞬時にスキャン。
- **🇯🇵 日本の個人情報に対応**: マイナンバー、運転免許証番号、住所、電話番号などの日本固有フォーマットを高精度に検出。
- **👮 商用グレード機能**: TOMLによるカスタムルール定義、インライン無視機能 (`# veil:ignore`)、商用レベルのレポート出力。
- **📊 レポート機能**: 機械処理可能な JSON 出力に加え、提出用としてそのまま使える HTML ダッシュボード (`--format html`) を生成可能。
- **🛡️ 堅牢な制御**: `--fail-score` によるCI合否判定、`WARN` レベルの柔軟な運用が可能。
- **⚡ Staged Scan**: `--staged` オプションでコミット予定のファイルだけを高速スキャン。`pre-commit` に最適。
- **📦 バイナリ/巨大ファイル対策**: バイナリファイルや1MB超の巨大ファイルは自動でスキップし、CIの詰まりや文字化けを防止。
- **🔧 完全設定可能 & 階層化**: `veil.toml` に加え、組織ごとの共通設定 (`VEIL_ORG_RULES`) を読み込む階層化ポリシー管理に対応。
- **💉 Stop the Bleeding (Baseline)**: 既存の技術的負債をスナップショット化し、"新規の漏洩" だけを確実に止める [Baseline Scanning](docs/baseline/usage.md) を標準搭載。

## Minimum Supported Rust Version (MSRV)
We support the latest stable Rust and **MSRV 1.82.0**.
- **Patch Policy**: Patch releases never bump MSRV.
- **Minor Policy**: Minor releases may bump MSRV (documented in release notes).

### Quick Install (Rust 開発者向け)

```bash
curl -fsSL https://raw.githubusercontent.com/mt4110/veil-rs/main/scripts/install.sh | sh
veil --version
```

### Nix

```bash
# 一時的に試す
nix run github:mt4110/veil-rs#veil -- --version

# ローカルでビルドだけしたい場合
nix build github:mt4110/veil-rs#veil
ls result/bin/veil
```

### ソースコードからビルド
```bash
git clone https://github.com/mt4110/veil-rs.git
cd veil-rs

# 開発環境に入る (推奨: 必要なRustバージョンやライブラリが揃います)
nix develop

cargo build --release
```

> [!TIP]
> **Check MSRV (1.82.0)**
> ```bash
> nix develop .#msrv
> ```


> [!IMPORTANT]
> **開発者向け: Nix環境の利用について**
> 本プロジェクトは `nix develop` 環境での開発を前提としています。
> システムの Rust バージョンが古い場合（例: 1.82.0以下）、最新の依存クレート（Rust 2024 Edition要求など）のビルドに失敗する可能性があります。
> 必ず `nix develop` を経由して、プロジェクトが指定する適切なツールチェーンを使用してください。

## 使い方

### 0. 初期セットアップ (推奨)
プロジェクトに合わせた最適な設定ファイルを対話的に作成します。

```bash
veil init --wizard
```

### 1. 安全性の確認 (推奨)
新しいルールを追加したり設定を変更した場合は、必ず構成チェックを行ってください。

```bash
veil config check
```

### 2. 基本スキャン
```bash
veil scan .
```

### 3. 脆弱性スキャン (Guardian)
ロックファイル (Cargo.lock, package-lock.json 等) を解析し、既知の脆弱性を検出します。

```bash
# 通常スキャン (高速)
veil guardian check

# 詳細表示 (OSVから詳細情報を取得・キャッシュ)
veil guardian check --osv-details

# オフラインモード (キャッシュのみ使用)
veil guardian check --osv-details --offline
```

> [!TIP]
> **キャッシュと更新**:
> 詳細情報は `~/.cache/veil/guardian/osv/vulns` (OS依存) に保存されます。
> CI等で強制的に最新情報を取得したい場合は `VEIL_OSV_FORCE_REFRESH=1` を設定してください。

### JSON 出力
```bash
veil scan . --format json
```

### HTML ダッシュボード (監査レポート用)
```bash
veil scan . --format html > report.html
open report.html
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
環境変数 `VEIL_ORG_RULES` に共通設定ファイルのパスを指定すると、各プロジェクトの `veil.toml` とマージされます（プロジェクト設定が優先）。

```bash
export VEIL_ORG_RULES=/etc/veil/org_policy.toml
# org_policy.toml で "fail_on_score = 50" を設定しておけば、全プロジェクトで厳格なチェックを強制可能
```

### 3. CI/CD インテグレーション
GitHub Actions や GitLab CI ですぐに使えるテンプレートを `examples/ci/` に用意しています。

**GitHub Actions の例:**
```yaml
- name: Veil Security Scan
  run: |
    # HTMLレポートを生成（アーティファクト保存用）
    veil scan . --format html > report.html

    # スコア80以上の検出があればCI失敗
    veil scan . --fail-score 80
    
    # または、変更されたファイルだけをチェック (Pull Request時など)
    # veil scan --staged
```


### 4. Integrations

Detailed guides for integrating `veil-rs` into your workflow:

*   **[pre-commit Framework](docs/integrations/pre-commit.md)**: Drop-in support for `.pre-commit-config.yaml`.
*   **[Native Git Hooks](docs/integrations/git-hook.md)**: Simple shell script for `.git/hooks`.
*   **[GitHub Actions](docs/integrations/github-actions.md)**: CI integration template.

---

## ライセンス
Apache 2.0 または MIT ライセンスのデュアルライセンスです。

> **Note**: 現在は MIT/Apache-2.0 のOSSとして提供していますが、将来のバージョンでエンタープライズ向けの高度な機能については、異なるライセンス体系や有償アドオンとして提供する可能性があります（v0.x系はOSSのまま維持されます）。私たちは持続可能なOSS開発のために、最適なモデルを模索しています。
