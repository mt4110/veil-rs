# 6. LSP Integration 詳細設計

## 6.1 目的

エディタ上でコード記述中にJP-PII/Secretを即時検知し、保存前に漏洩を止める。

## 6.2 crate構成

```text
crates/veil-lsp/
  Cargo.toml
  src/main.rs
  src/server.rs
  src/config.rs
  src/document_store.rs
  src/diagnostics.rs
  src/code_actions.rs
  src/range_map.rs
```

## 6.3 依存

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
tower-lsp = "0.20"
veil-core = { path = "../veil-core" }
veil-config = { path = "../veil-config" }
serde = { version = "1", features = ["derive"] }
```

## 6.4 Capabilities

```rust
ServerCapabilities {
    text_document_sync: Incremental,
    diagnostic_provider: Some(...),
    code_action_provider: Some(true),
    hover_provider: Some(true),
    workspace: Some(...),
}
```

## 6.5 Document Store

```rust
struct DocumentState {
    uri: Url,
    text: Rope,
    version: i32,
    last_scan_revision: u64,
}

struct DocumentStore {
    docs: DashMap<Url, DocumentState>,
}
```

## 6.6 Diagnostics変換

### Finding → Diagnostic
| Finding | Diagnostic |
|---|---|
| path | uri |
| line_number | range.start.line |
| `LspFinding.utf16_range: Range` | range |
| severity | DiagnosticSeverity |
| rule_id | code |
| score | data.score |

### Severity mapping
| Veil | LSP |
|---|---|
| Critical | ERROR |
| High | ERROR |
| Medium | WARNING |
| Low | INFORMATION |

## 6.7 Debounce

- `didChange`ごとに即scanしない。
- 150〜300ms debounce。
- 同一documentに対して古いscan結果は捨てる。
- 1ファイル最大サイズを設定。大きすぎる場合はdiagnosticで「skipped」。

## 6.8 Range / Span contract

- `masked_snippet` は表示専用であり、Diagnostic rangeやCodeAction edit範囲に使わない。
- Coreは normalized span を original byte span に戻し、LSP層で UTF-16 range に変換する。
- LSP Diagnosticの `data` は `ruleId`, `score`, `grade`, `maskedSnippet`, `actions` のみ。raw値は載せない。
- `MAX_FILE_SIZE` の skipped diagnostic はFinding由来ではないため、先頭 `0:0` range の synthetic diagnostic としてLSP層で生成する。

```rust
pub struct Position {
    pub line: u32,
    pub character: u32, // UTF-16 code units, LSP compatible
}

pub struct Range {
    pub start: Position,
    pub end: Position,
}

pub struct OriginalSpan {
    pub byte_start: usize,
    pub byte_end: usize,
}

pub struct LspFinding {
    pub safe: SafeFindingApiV1,
    pub original_span: OriginalSpan, // internal only; not serialized to LSP data
    pub utf16_range: Range,
}
```

`utf16_range` を唯一のDiagnostic/CodeAction範囲SOTにする。`masked_snippet` からrangeを復元してはならない。

## 6.9 Code Actions

| Action | 内容 |
|---|---|
| Mask value | 該当rangeを `<REDACTED>` へ置換 |
| Partial mask | 末尾4桁のみ保持 |
| Add inline ignore | 言語別コメント構文で `veil:ignore=rule.id` を挿入 |
| Open rule docs | rule説明を開く |

### ignore comment syntax

| language/filetype | comment | 例 |
|---|---|---|
| Rust/Go/JS/TS/Java/C/C++ | `//` | `// veil:ignore=pii.jp.mynumber.keyword` |
| Python/Ruby/Shell/YAML/TOML | `#` | `# veil:ignore=...` |
| HTML/XML | `<!-- -->` | `<!-- veil:ignore=... -->` |
| SQL | `--` | `-- veil:ignore=...` |
| JSON | なし | inline ignore actionを出さない |

### 安全ルール
- CodeActionは自動適用しない。
- raw matched_content は `data` に載せない。
- Mask previewはエディタ側のdiffで表示。

## 6.10 LSP起動コマンド

```bash
veil lsp --preset fintech-jp
```

Neovim例:
```lua
vim.lsp.start({
  name = "veil-lsp",
  cmd = { "veil", "lsp", "--preset", "fintech-jp" },
  root_dir = vim.fn.getcwd(),
})
```

## 6.11 実装タスク

- [x] `crates/veil-lsp` workspace追加。
- [x] `tower-lsp` server実装。
- [x] `scan_content` をLSPから呼び出す。
- [x] `didOpen` / `didChange` で `publishDiagnostics` を送る。
- [x] `didClose` でdiagnosticsをclearする。
- [x] `didChange` のscanを debounce し、同一documentの古い結果をpublishしない。
- [x] open document が最大サイズ超過のとき `MAX_FILE_SIZE` skipped diagnostic を publish する。
- [x] UTF-8 byte offset → UTF-16 LSP range変換はCoreの`Finding.utf16_range`に集約し、LSP Diagnosticでは再計算しない。
- [ ] `codeAction` for mask/ignore。
- [ ] 言語別ignore comment registryを実装。
- [ ] JSON等コメント不可ファイルではinline ignore actionを非表示。
- [ ] fixtureでNeovim想定のdiagnosticテスト。
- [ ] `veil lsp` をCLIに追加。


## 6.11 v4 Range SOT

LSPのRange型は以下に固定する。

```rust
pub struct Position {
    pub line: u32,
    pub character: u32, // UTF-16 code units
}

pub struct Range {
    pub start: Position,
    pub end: Position,
}

pub struct LspFinding {
    pub safe: SafeFindingApiV1,
    pub original_span: FindingSpan,
    pub utf16_range: Range,
}
```

`line`, `utf16_start`, `utf16_end` の単独3値モデルは使用しない。`masked_snippet` からrangeを復元してはならない。
