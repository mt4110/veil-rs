# 2. コンポーネント別詳細設計

## 2.1 veil-core

### 責務
- RulePack読み込み・検証
- ファイルシステム走査
- JPテキスト正規化
- PII / Secret検出
- マスキング
- スコアリング
- Baseline適用
- Evidence Pack検証

### Public API案
```rust
pub fn scan_path(root: &Path, rules: &[Rule], config: &Config) -> ScanResult;
pub fn scan_content(content: &str, path: &Path, rules: &[Rule], config: &Config) -> Vec<Finding>;
pub fn apply_masks(content: &str, ranges: Vec<Range<usize>>, mode: MaskMode, placeholder: &str) -> String;
pub fn verify_evidence_pack(path: &Path, options: VerifyOptions) -> Result<VerifyResult, VerifyError>;
pub fn normalize_jp_text(input: &str, policy: NormalizationPolicy) -> NormalizedText;
pub fn validate_jp_identifier(kind: JpIdentifierKind, raw: &str) -> ValidationOutcome;
```

### I/O
| 入力 | 出力 | 備考 |
|---|---|---|
| `Path`, `Config`, `Vec<Rule>` | `ScanResult` | 並列scan。unsupported/oversized binary は expected skip。text/log/source の oversize は coverage gap として incomplete |
| `&str`, `Path`, `Config` | `Vec<Finding>` | LSP/Filter/Unit testで使用 |
| `Evidence ZIP`, `VerifyOptions` | `VerifyResult` | ZipSlip/Bomb/Hash/Schema/Token leak検査 |

### 内部モジュール案
| モジュール | 役割 |
|---|---|
| `scanner` | WalkBuilder + rayon による高速走査 |
| `scanner::jp_normalize` | 全角/半角/ハイフン/スペース正規化 |
| `rules` | RulePack読込、manifest順序、validator関連付け |
| `validators::jp` | MyNumber / Luhn / 住所/電話などの検証器 |
| `masking` | Redact/Partial/Plain。ただしplainはCLI明示のみ |
| `baseline` | 既存Findingのsnapshot化・抑制 |
| `verify` | Evidence Pack検証 |

## 2.2 veil-config

### 責務
- `veil.toml` / `veil.ci.toml` / `VEIL_ORG_CONFIG` / preset の合成
- 安全境界の検証
- Zero-Configの推論結果を Config に落とす

### Config Layer順序
```text
built-in defaults
→ preset template
→ org config (VEIL_ORG_CONFIG)
→ repo config (veil.toml)
→ CLI flags
```

### API案
```rust
pub fn load_config_layers(input: LoadConfigInput) -> Result<ConfigLayers>;
pub fn apply_preset(base: Config, preset: PresetId) -> Result<Config>;
pub fn validate_config(config: &Config) -> Result<()>;
pub fn explain_effective_config(config: &ConfigLayers) -> EffectiveConfigExplanation;
```

### PresetId
```rust
pub enum PresetId {
    StandardJp,
    FintechJp,
    GovJp,
    SiVendorJp,
    LogsJp,
}
```

## 2.3 veil-cli

### 責務
- コマンド引数を解析し、Coreへ委譲
- stdout/stderr purityを保証
- Interactive CLI状態制御
- Evidence verify
- LSP起動コマンドの提供

### 主要コマンド
| コマンド | 責務 |
|---|---|
| `veil init` | Zero-Config初期化。環境検出、プリセット選択、CI雛形生成 |
| `veil scan` | ローカル/CIスキャン |
| `veil scan --interactive` | 対話式マスキング/無視/スキップ |
| `veil scan --preset fintech-jp` | プリセット即時適用 |
| `veil verify evidence.zip` | Evidence Pack検証 |
| `veil lsp` | Language Server起動 |
| `veil ui` | Local Audit UI起動（または `veil-pro` ラップ） |

## 2.4 veil-lsp（新規）

### 責務
- LSP標準に準拠し、エディタ上にPII/Secret警告を表示
- 保存前に検知し、シフトレフトを実現
- オフライン・ローカルのみ

### 入出力
| LSPイベント | 処理 | 出力 |
|---|---|---|
| `initialize` | Config/Preset読み込み | capabilities |
| `textDocument/didOpen` | buffer scan | diagnostics |
| `textDocument/didChange` | debounce scan | diagnostics |
| `textDocument/codeAction` | mask/ignore候補 | workspace edit |
| `workspace/didChangeConfiguration` | config再読込 | diagnostics再計算 |

## 2.5 veil-pro Local Server

### 責務
- `127.0.0.1` のみでSvelte UIを配信
- Token認証 / CSP / no-store / no-referrer
- Scan API, Evidence Export API
- RunCache TTL/容量制御

### I/O
| Endpoint | 入力 | 出力 |
|---|---|---|
| `GET /api/me` | Bearer token | `AuthContext` |
| `GET /api/projects` | なし | `ProjectsResponse` |
| `POST /api/scan` | `ScanRequest` | `ScanResponse` |
| `GET /api/runs/{id}` | run_id | `RunMetaResponse` |
| `GET /api/runs/{id}/evidence.zip` | run_id | ZIP |
| `POST /api/baseline` | `BaselineRequest` | `BaselineResponse` |
| `GET /api/policy` | なし | `PolicyResponse` |
| `GET /api/doctor` | なし | `DoctorResponse` |

## 2.6 Svelte UI

### 責務
- ユーザー操作面
- Findings可視化
- Baseline作成
- Policy表示
- Evidence ZIP export

### 画面構成
| 画面 | 主CTA | 補助CTA |
|---|---|---|
| Projects | Add/Open Project | Recent |
| Scan | Start Scan | Preset select |
| Findings | Export Evidence ZIP | Filter / Search |
| Baseline | Create Baseline | View baseline |
| Policy | View Effective Config | Copy diagnostics |
| Doctor | Export Doctor Snapshot | Copy env info |

## 2.7 Evidence Pack

### 内容
```text
report.html
report.json
effective_config.toml
run_meta.json
veil.baseline.json  // optional; baseline未使用時は含めない
```

### 契約
- ZIP内パス固定
- raw secretを含めない
- `run_meta.json` raw bytes SHA256を外部アンカーにできる
- `veil verify` は構造、schema、sha256、token leak、ZipSlip/Bombを検査
