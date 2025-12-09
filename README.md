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

## インストール

```bash
# ソースコードからビルド
git clone https://github.com/takem1-max64/veil-rs.git
cd veil-rs
cargo install --path crates/veil-cli --bin veil
```

## 使い方

### 0. 初期セットアップ (推奨)
プロジェクトに合わせた最適な設定ファイルを対話的に作成します。

```bash
veil init --wizard
```

### 1. 基本スキャン
```bash
veil scan .
```

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

*   **`schemaVersion`**: Always check this field first (`veil-v1`). If it differs, the structure may have changed.
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
    # スコア80以上の検出があれば失敗させる（CI用）
    veil scan . --format html > report.html
    # スコア80以上の検出があれば失敗させる（CI用）
    veil scan . --fail-score 80
    # または、変更されたファイルだけをチェック (Pull Request時など)
    # veil scan --staged
```


### 4. Git フック (pre-commit)
`pre-commit` を使用して、コミット前に自動スキャンを行うことができます。
`.pre-commit-config.yaml` に以下を追加してください：

```yaml
repos:
  - repo: local
    hooks:
      - id: veil-scan
        name: veil-scan
        entry: veil scan
        language: system
        types: [text]
        exclude: '\.git/|\.png$|\.jpg$'
```

## ライセンス
Apache 2.0 または MIT ライセンスのデュアルライセンスです。

> **Note**: 現在は MIT/Apache-2.0 のOSSとして提供していますが、将来のバージョンでエンタープライズ向けの高度な機能については、異なるライセンス体系や有償アドオンとして提供する可能性があります（v0.x系はOSSのまま維持されます）。私たちは持続可能なOSS開発のために、最適なモデルを模索しています。
