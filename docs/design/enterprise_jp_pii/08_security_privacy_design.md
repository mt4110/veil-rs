# 8. Security / Privacy 設計

## 8.1 非交渉要件

- ソースコード、PII、スキャン結果を外部APIへ送信しない。デフォルトでは外部通信を一切行わない。
- Local UIは `127.0.0.1` のみ。
- `0.0.0.0` bindを許可しない。
- raw secretをUI API/Evidenceに含めない。
- stdout/stderr purityを維持。

## 8.2 Local UI Security

### Headers
```http
Content-Security-Policy: default-src 'self'; img-src 'self' data:; style-src 'self'; script-src 'self'; connect-src 'self';
Referrer-Policy: no-referrer
Cache-Control: no-store
X-Frame-Options: DENY
X-Content-Type-Options: nosniff
```

### Token設計
- 起動時に32bytes乱数を生成。
- URLは `http://127.0.0.1:{port}/#token=...`。
- fragmentはHTTPに送信されない。
- SvelteはhashからsessionStorageへ移し、URLから消す。
- APIは `Authorization: Bearer <token>` のみ。

### 禁止
- CDN依存
- inline script/style
- eval
- `{@html}`
- raw `matched_content` のUI送信

## 8.3 Path安全

### validate_safe_path
- `..` tokenを拒否。
- canonicalizeして project root prefix を確認。
- symlink脱出を拒否。
- baseline保存先はプロジェクト配下固定。

## 8.4 Evidence安全

### Evidence ZIP
- 固定エントリ名のみ。
- ZIP内に `../` / 絶対パス / drive letter 禁止。
- token leakage check。
- `run_meta.json` raw bytes SHA256を外部アンカーにする。

### 表現
- tamper-proofと言わない。
- tamper-evident: 改ざんがあれば検知できる。

## 8.5 LSP安全

- LSPはローカルプロセス。
- document textを外部送信しない。
- diagnosticsには masked snippet か rule_id のみ。
- code actionはユーザー確認後のみ。

## 8.6 Enterprise opt-in network boundary

| 機能 | デフォルト | 有効化条件 | 外部送信禁止 |
|---|---|---|---|
| SSO | 無効 | `VEIL_PRO_ENABLE_SSO=1` + Enterprise設定 | source/PII/findings/Evidence |
| Remote RulePack | 無効 | `core.allow_remote_rules=true` + `VEIL_ALLOW_NETWORK=1` | source/PII/findings/Evidence |

Remote RulePack は署名検証に失敗した場合は読み込まない。Air-gapでは使用しない。
v1のoffline検証は `trust_model = "pinned_digests"` と `digest_algorithm = "sha256"` を対象にし、
pack metadata とRulePack file digestの正準SHA256が `pinned_digests` に含まれる場合のみ読み込む。
`pinned_keys` / `tofu` は未実装のため fail closed とする。
RulePack更新は `implementation/rulepack_update_flow.md` の通り、candidate packをstagingで検証し、
active `rules_dir` へatomic promoteする。自動ネットワーク更新はv1では行わない。

## 8.7 Config安全

- config fileで `mask_mode = plain` 禁止。
- `--unsafe` / `--mask-mode plain` はCLIで明示指定のみ。
- regexは長さ上限とコンパイル検証。
- ReDoS危険パターンを検知するlintを追加可能。

## 8.8 Threat Model

| 脅威 | 対策 |
|---|---|
| UI token漏洩 | fragment token, no-referrer, no-store |
| XSS | strict CSP, no inline, Svelte escape, no raw HTML |
| path traversal | canonical prefix check |
| ZIP slip/bomb | verify bounds + fixed entries |
| CI誤通過 | incomplete scan exit 2 |
| raw secret漏洩 | masked_snippet only |
| 版ブレ | SOT check scripts / bundle |


## 8.7 privacy.networkMode contract

Local API / Evidence `run_meta.privacy` は `networkMode` を使う。

| networkMode | 意味 |
|---|---|
| `local-only` | デフォルト。外部通信なし。 |
| `enterprise-opt-in` | SSOまたは署名付きRemote RulePack取得を明示有効化。ソース/PII/findings/Evidence送信は禁止。 |

`privacy.network` 固定文字列は使わない。Enterprise opt-in時に嘘になるためである。
