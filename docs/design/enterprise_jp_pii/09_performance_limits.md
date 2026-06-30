# 9. Performance / Limits 設計

## 9.1 目標

| シナリオ | 目標 |
|---|---:|
| staged scan | < 1s |
| small repo 1k files | < 3s |
| medium repo 50k files | < 30s |
| CI default | fail fast, no hang |
| LSP single document | < 150ms p95 |

## 9.2 性能方針

- `ignore::WalkBuilder` で `.gitignore` 尊重。
- built-in heavy dirsを除外。
- ファイル収集は上限付き。
- ファイルscanは rayon 並列。
- regexはRulePack読み込み時にcompile済み。
- LSPはdocument単位でscanし、ファイルシステム全走査しない。

## 9.3 Built-in ignores

```text
node_modules, target, dist, build, vendor, .git,
.direnv, .venv, venv, __pycache__, .cache,
.pytest_cache, .mypy_cache, .ruff_cache, .tox,
coverage, .nyc_output, .next, .nuxt, .svelte-kit,
.terraform, .terragrunt-cache
```

## 9.4 Limits

| 設定 | default | hard max | 不完全時 |
|---|---:|---:|---|
| `core.max_file_size` | 1MB | 500MB | text/log/source超過は incomplete / exit 2、binary超過は expected skip |
| `core.max_file_count` | 1,000,000 | 1,000,000 | exit 2 |
| `output.max_findings` | 1000 | 100,000 | exit 2 / incomplete |
| UI RunCache runs | 20 | config不可 | evict |
| UI RunCache bytes | 100MB | config不可 | evict |
| UI RunCache TTL | 30min | config不可 | 410 Gone |

## 9.5 Skip / incomplete分類

| skip種別 | 例 | 完全性 | CI扱い |
|---|---|---|---|
| expected skip | `.gitignore`, `[core] ignore`, built-in heavy dirs | complete | Exit 0/1の通常判定 |
| binary skip | 画像/圧縮/実行ファイル等 | complete | Exit 0/1の通常判定 |
| coverage skip | text/log/sourceのmax_file_size超過 | incomplete | Exit 2 |
| traversal/output limit | max_file_count, max_findings | incomplete | Exit 2 |
| read/config/rule error | permission, invalid regex | error | Exit 2 |

## 9.6 CI対策

- Limit到達は success扱いにしない。
- JSON/HTML stdout purityを壊さない。
- warningはstderr。
- `--staged` はGit diffから対象ファイルだけscan。

## 9.7 LSP性能

- debounce 150〜300ms。
- document size > 1MB はskip diagnostic。
- RulePackをworkspaceで共有。
- 前回document hashが同一なら再scanしない。
- ファイル保存時のみfull line contextを追加。

## 9.8 Benchmark計画

| ベンチ | 内容 |
|---|---|
| `scan_benchmark` | 1MB/10MB/100MB synthetic |
| `jp_pii_normalize` | 全角半角変換 + span map |
| `rulepack_compile` | presetごとのregex compile時間 |
| `lsp_latency` | didChange→diagnostic p95 |
| `evidence_zip` | 1k/10k findings zip生成 |


## 9.8 fail-on-findings comparison

`--fail-on-findings N` は `effectiveFindings >= N` で判定する。`N=0` は設定エラー。`effectiveFindings` は baseline suppress 後の件数であり、`totalFindings` ではない。
