# PR-56 — Issue #47 PR-C (idna/url) Remediation — deterministic evidence loop

## ✅ Meta (fill by checklist)
- [ ] PR URL: <paste PR URL here>
- [x] Issue URL: https://github.com/mt4110/veil-rs/issues/47
- [x] Branch: feature/pr56-issue47-prc-idna-url-remediation
- [x] Base (main) at kickoff: 8018f93
- [x] Implementation HEAD (no code changes in PR-C): 8018f93
- [x] Latest prverify evidence: docs/evidence/prverify/prverify_20260211T120918Z_8018f93.md
- [x] cockpit check: PASS (see log snippet below)

---

## ✅ Summary
**PR-C（idna/url）について、依存更新は不要**であることを、
決定論スナップショット + prverify 証拠で固定する。

- `url` は **2.5.4**（target: >= 2.5.4）
- `idna` は **1.1.0**（target: >= 1.0.3）
- よって **このPRは “更新” ではなく “既に安全であることの証明”** を目的とする（再発防止＝証拠ループ）。

---

## ✅ Context
- PR55 merged ✅: pre-commit guard (no *.sh) + docs file-url hygiene
- Issue #47 OPEN: dependabot remediation series
- PR-B done as PR54 ✅
  - bytes: already at 1.11.1
  - rsa: removed from dependency tree
  - Evidence: docs/evidence/prverify/prverify_20260211T041814Z_6c4a3bd.md

---

## 🎯 Objective (PR56 / PR-C)
Issue #47 の PR-C（idna/url）を **再発しない形**（最小更新 or 無変更の証明 + 決定論 + 証拠ループ）で main に入れる。

---

## 🔒 Normative security target (source-of-truth)
Advisory: RUSTSEC-2024-0421 / GHSA-h97m-ww89-6jmq / CVE-2024-12224  
Remedy (normative):
- `idna` を直接依存している場合: **Upgrade to `idna >= 1.0.3`**
- `url` 経由で `idna` を引いている場合: **Upgrade to `url >= 2.5.4`**
Refs:
- RustSec: https://rustsec.org/advisories/RUSTSEC-2024-0421
- GHSA: https://github.com/advisories/GHSA-h97m-ww89-6jmq

---

## ✅ Always Run Contract (PR56)
- SOT: this file
- plan/task: docs/pr/PR-56-issue-47-prc-idna-url-remediation/{implementation_plan.md,task.md}
- `cargo test --workspace` PASS
- `nix run .#prverify` PASS
- evidence archived: docs/evidence/prverify/prverify_20260211T120918Z_8018f93.md
- cockpit check PASS

---

## 🧭 Deterministic Inputs (snapshots)

### A) Issue snapshot (machine-extracted)
Commands:
- `gh issue view 47 --json title,body,state,url`
- `gh issue view 47 --comments --json comments | jq -r '.comments[].body' | sed -n '1,200p'`

Paste outputs (verbatim):

- [x] Issue JSON (title/body/state/url):
```json
{
  "body": "# Dependabot Triage Report - 2026-02-10\\n\\n## Executive Summary\\nThis document snapshots the state of 6 active Dependabot alerts as of 2026-02-10.\\nThe goal is to remediate these vulnerabilities through a series of small, verifiable PRs, prioritizing low-risk fixes first.\\n\\n## Active Vulnerabilities (6)\\n\\n| ID | Severity | Ecosystem | Package | Type | Version | Fixed In | Breaking Risk | Remediation Plan | Notes |\\n|:---|:---|:---|:---|:---|:---|:---|:---|:---|:---|\\n| GHSA-434x-w66g-qw3r | Medium | Rust | `bytes` | Transitive | 1.11.0 | 1.11.1 | Low | **PR-B** (`cargo update -p bytes`) | Integer overflow in reserve. Safe for most uses but should update. |\\n| GHSA-9c48-w39g-hm26 | Low | Rust | `rsa` | Transitive | 0.9.9 | 0.9.10 | Low | **PR-B** (`cargo update -p rsa`) | Panic on prime=1. Rarely hit but easy fix. |\\n| GHSA-h97m-ww89-6jmq | Medium | Rust | `idna` | Transitive | 0.5.0 | 1.0.0 | **Medium/High** | **PR-C** (`cargo update -p idna` or `url`) | Punycode issue. Verify `url` crate compatibility (Major version bump risk). |\\n| GHSA-j39j-6gw9-jw6h | Low | Rust | `git2` | Direct + Lockfile | 0.19.0 | 0.20.4 | High | **PR-D** (Update `veil-cli`) | Potential breaking changes in `git2` 0.20. Requires code changes. (Alert covers both direct and lockfile entries). |\\n| GHSA-g98v-hv3f-hcfr | Low | Rust | `atty` | Direct (`veil-cli`) | 0.2.14 | *Unmaintained* | High | **PR-E** (Replace dependency) | `atty` is unmaintained using unaligned reads on Windows. Replace with `is-terminal` or `std::io::IsTerminal`. |\\n\\n## Remediation Strategy\\n\\n### 1. Low Risk / Patch Updates (PR-B)\\nFocus on simple `cargo update -p <package>` for transitive dependencies with available fixes and low breaking risk.\\n- **Scope**: `bytes`, `rsa`\\n- **Verification**: `cargo test --workspace`, `nix run .#prverify`\\n\\n### 2. Medium Risk / Minor Updates (PR-C)\\nUpdate dependencies that might have minor breaking changes or require ensuring compatibility with parent crates (e.g., `url` -> `idna`).\\n- **Scope**: `idna` (via `url`?)\\n- **Verification**: Ensure no `url` related breakages.\\n\\n### 3. High Risk / Breaking Changes (PR-D & PR-E)\\nHandle direct dependencies that require code changes or major version bumps.\\n- **Scope**: `git2` (0.19 -> 0.20), `atty` (Replace)\\n- **Verification**: Full manual verification of CLI functionality.\\n\\n## Definition of Done\\n- [ ] All 6 alerts are closed or dismissed with valid reason.\\n- [ ] Each PR passes `nix run .#prverify`.\\n- [ ] Documentation updated to reflect changes.\\n",
  "state": "OPEN",
  "title": "security: remediate 6 dependabot alerts (triage 2026-02-10)",
  "url": "https://github.com/mt4110/veil-rs/issues/47"
}
```

* [x] First 200 lines of comments (joined):

```text
PR-B (bytes/rsa) done as PR54.
- bytes: already at 1.11.1
- rsa: removed from dependency tree
- Evidence: docs/evidence/prverify/prverify_20260211T041814Z_6c4a3bd.md

PR: <paste PR url here>
All dependabot alerts remediated via PR series (PR-B..PR-E).
Reopened: PR54 only covers PR-B (bytes/rsa) verification.

Status:
- PR-B done as PR54: https://github.com/mt4110/veil-rs/pull/54
  - bytes: already at 1.11.1
  - rsa: removed from dependency tree
  - Evidence: docs/evidence/prverify/prverify_20260211T041814Z_6c4a3bd.md

Next: proceed with remaining PRs in the series (PR-C..PR-E) per Issue #47 plan.
PR-B completed as PR54.

- PR: https://github.com/mt4110/veil-rs/pull/54
- bytes: already at 1.11.1
- rsa: removed from dependency tree
- Evidence: docs/evidence/prverify/prverify_20260211T041814Z_6c4a3bd.md

Next: proceed with remaining items (PR-C..PR-E) per Issue plan.
```

---

### B) Dependency snapshot (BEFORE)

Commands:

* `cargo tree -i idna`
* `cargo tree -i url`
* `cargo tree -p url`
* `cargo tree -p idna`

Key outputs:

* [x] `cargo tree -i idna`

```text
idna v1.1.0
├── email_address v0.2.9
│   └── veil-config v0.17.0 (<repo>/crates/veil-config)
│       └── veil-cli v0.17.0 (<repo>/crates/veil-cli)
└── url v2.5.4
    ├── git2 v0.19.0
    │   ├── veil-cli v0.17.0 (<repo>/crates/veil-cli) (*)
    │   └── veil-config v0.17.0 (<repo>/crates/veil-config) (*)
    ├── veil-cli v0.17.0 (<repo>/crates/veil-cli) (*)
    ├── veil-config v0.17.0 (<repo>/crates/veil-config) (*)
    └── veil-core v0.17.0 (<repo>/crates/veil-core)
        ├── veil-cli v0.17.0 (<repo>/crates/veil-cli) (*)
        └── veil-guardian v0.17.0 (<repo>/crates/veil-guardian)
            └── veil-cli v0.17.0 (<repo>/crates/veil-cli) (*)
```

* [x] `cargo tree -i url`

```text
url v2.5.4
├── git2 v0.19.0
│   ├── veil-cli v0.17.0 (<repo>/crates/veil-cli)
│   └── veil-config v0.17.0 (<repo>/crates/veil-config)
│       └── veil-cli v0.17.0 (<repo>/crates/veil-cli) (*)
├── veil-cli v0.17.0 (<repo>/crates/veil-cli) (*)
├── veil-config v0.17.0 (<repo>/crates/veil-config) (*)
└── veil-core v0.17.0 (<repo>/crates/veil-core)
    ├── veil-cli v0.17.0 (<repo>/crates/veil-cli) (*)
    └── veil-guardian v0.17.0 (<repo>/crates/veil-guardian)
        └── veil-cli v0.17.0 (<repo>/crates/veil-cli) (*)
```

* [x] `cargo tree -p url`

```text
url v2.5.4
```

* [x] `cargo tree -p idna`

```text
idna v1.1.0
```

---

## 🧠 Decision record (path selection)

* [x] Path-A: `idna` is pulled via `url` → require `url >= 2.5.4`
* [ ] Path-B: `idna` is direct dependency → require `idna >= 1.0.3`

Rationale:

* Snapshot shows `url v2.5.4` already present and pulling `idna v1.1.0`.
* Both satisfy the normative targets (`url >= 2.5.4`, `idna >= 1.0.3`).
* Therefore no dependency update is required for PR-C; remediation is to freeze evidence.

---

## 🛠️ Change Summary (AFTER)

### A) What changed (versions)

No dependency changes in PR-C (already compliant).

* url BEFORE: 2.5.4
* url AFTER : 2.5.4
* idna BEFORE: 1.1.0
* idna AFTER : 1.1.0

### B) Dependency snapshot (AFTER)

Same as BEFORE (no changes applied).

### C) Files changed (sanity)

Expected: docs/pr/* + docs/evidence/* only.

`git diff --name-only`:

```text
docs/evidence/prverify/prverify_20260211T120918Z_8018f93.md
docs/pr/PR-56-issue-47-prc-idna-url-remediation.md
docs/pr/PR-56-issue-47-prc-idna-url-remediation/implementation_plan.md
docs/pr/PR-56-issue-47-prc-idna-url-remediation/task.md
```

---

## ✅ Verification Evidence

* prverify PASS:

  * docs/evidence/prverify/prverify_20260211T120918Z_8018f93.md

* cockpit check PASS (snippet):

```text
COCKPIT_RFv1: RESULT=PASS STEP=check
✓ cockpit check passed
```

---

## ✅ “Why this closes PR-C”

PR-C の目的は「idna/url の脆弱性を塞ぐ」ことだが、
現時点の依存状態がすでに `url v2.5.4` + `idna v1.1.0` を満たしているため、
PR-C は依存更新ではなく “既に安全である証拠の固定” を行う。

---

## Follow-ups

* PR作成後:

  * Meta の PR URL を埋めてチェック
  * Issue #47 に “PR-Cは更新不要（既に安全）” のコメント + 証拠リンクを残す
* もし GitHub の dependabot alert が残る場合:

  * 再スキャン待ち/手動トリガーの可能性があるので、Issue に状況を書き残す（証拠はこのSOT）
