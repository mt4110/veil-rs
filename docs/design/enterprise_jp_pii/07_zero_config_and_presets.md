# 7. Zero-Config & Presets 設計

## 7.1 目的

ユーザーに `.toml` を書かせず、初回から日本向けPII検知を稼働させる。

## 7.2 Preset一覧

| Preset | 対象 | 方針 |
|---|---|---|
| `standard-jp` | 一般 | secrets + JP PII 基本 |
| `fintech-jp` | 金融/決済 | MyNumber, bank, cardを強化 |
| `gov-jp` | 官公庁/自治体 | MyNumber, 住所, 氏名, 電話を強化 |
| `si-vendor-jp` | SIer/受託 | Evidence, baseline, CI導入重視 |
| `logs-jp` | ログ監査 | JSONL/log特化、PII key context |

## 7.3 Preset内部表現

```rust
pub struct Preset {
    pub id: PresetId,
    pub display_name: &'static str,
    pub description: &'static str,
    pub config_patch: Config,
    pub rule_packs: Vec<RulePackRef>,
    pub warnings: Vec<PresetWarning>,
}
```

## 7.4 `veil init` 環境判別

### 入力信号
| シグナル | 推論 |
|---|---|
| `Cargo.toml` | Rust project |
| `package.json` | Node project |
| `.github/workflows` | GitHub Actions |
| `*.log`, `logs/` | logs-jp候補 |
| `README.md`に日本語 | jp presets優先 |
| `payments`, `billing`, `kyc`, `account` dirs | fintech-jp候補 |
| `terraform`, `infra` dirs | secrets重視 |

## 7.5 出力veil.toml最小化

Zero-configでは冗長なTOMLを書かない。

```toml
[core]
preset = "fintech-jp"

[output]
mask_mode = "redact"
```

または presetを設定ファイルに書かず、CLIで指定する設計も可能。

## 7.6 `veil scan --preset` の解決順序と説明責任

```text
builtin defaults
→ preset
→ VEIL_ORG_CONFIG
→ repo veil.toml
→ CLI flags
```

CLIの `--preset` は “最終上書き” ではなく、**base layer**として適用する。組織/リポジトリで上書き可能。これは一般的なCLI直感とズレるため、以下を必須とする。

- `veil config explain` は layerごとの差分を表示する。
- `--preset` で入った値を repo config / org config が上書きした場合、stderr に1行で説明する。
- explicit CLI flags（例: `--fail-on-score`, `--format`）は最終 layer として repo config より強い。
- CI軽量化は `minimal-ci` presetではなく、`mode = ci`, `--staged`, fail flags, scope制限で表現する。

## 7.7 テンプレ生成

```bash
veil init --preset fintech-jp
veil init --preset logs-jp --ci github
veil init --wizard
```

### Wizard質問
1. ソースコードを外部送信できるか → 常にNo前提で説明
2. 主なデータ種別 → PII / Secret / Logs / Payment
3. CIでfailさせたい閾値 → score/severity/findings
4. 既存負債があるか → baseline案内
5. UI/Evidenceが必要か → `veil ui`案内

## 7.8 プリセットTOML例

`templates/presets/fintech-jp.toml` を参照。


## 7.8 v4 Rule override contract

Preset TOML内のrule overrideは `enabled` と `base_score` のみを使う。`severity` は互換migration専用であり、新規presetには書かない。
