# Changelog

All notable changes to this project will be documented in this file.

## [0.22.1] - Unreleased

- Narrowly suppress lowercase technical port/name-mapping prose while retaining ordinary person labels and other findings.
- Distinguish a limited Rust `matches!` UID-accessor comparison from a customer/member ID value; retain labeled IDs, including values on the same line.
- Add synthetic regressions for both false positives and detection-preservation boundaries. Test/loopback IPv4 addresses are not blanket-excluded.
- 氏名・顧客／会員IDの誤検出2件を限定修正。通常の個人情報、同じ行の別ID、テスト用IPv4の検出を回帰検査で確認します。
- Align CLI/core/config/LSP, Cargo.lock and Nix metadata to 0.22.1; correct release-status documentation. External dependencies are unchanged.
- This is an unpublished preparation candidate. Published 0.22.0 assets remain unchanged. Linux x86_64 musl and macOS ARM64 only; macOS remains an ad-hoc-signed trial asset with Developer ID, notarization and downloaded Gatekeeper behavior unverified.

## [0.22.0] - 2026-10-16

[0.22.0](https://github.com/mt4110/veil-rs/releases/tag/v0.22.0) is published. The signed tag and both ZIPs with SHA256SUMS were verified, including anonymous public downloads. Bundled documents are build-time snapshots; later documentation changes do not replace these assets.


- Install from the extracted ZIP without Rust, sudo, network access or automatic shell-file edits; retain prior versions and refuse unmanaged CLI conflicts.
- Japanese/English help for the 19 public pages; explicit `--lang` overrides VEIL_LANG and locale. Reports, schemas, command identifiers and runtime messages retain existing contracts.
- Bilingual installation and first-file → staged-content → repair/restage → next-use guides, included in the archive and linked from help.
- Native archive acceptance exercises both languages, new shells, exit 0/1/2, ignored/binary/empty-stage cases and source/index preservation using synthetic data. This is not independent-user or real-development reuse evidence.
- ZIPからのローカル導入、日英help・導入案内、初回から次回までの合成検査を追加。検出ルール・秘匿・不完全検査・フックの安全条件は保持します。
- Linux x86_64 musl / macOS ARM64 only. macOS remains an ad-hoc-signed trial asset; Developer ID, notarization and downloaded Gatekeeper behavior remain unverified. Published 0.21.0 and rc.4 assets remain unchanged.

## [0.21.0] - 2026-10-16

[0.21.0](https://github.com/mt4110/veil-rs/releases/tag/v0.21.0) is published as the latest stable release. Published rc.4 remains available as a prerelease. Versions and documentation changed from rc.4; product code and external dependencies did not.

0.21.0正式版を公開しました。rc.4は公開済みプレリリースとして保持しています。rc.4から製品コード・外部依存を変更せず、版番号と文書を揃えています。

### Changes since v0.20.0 / v0.20.0からの変更

- Local OSS CLI: preserve config/core/cli responsibilities, standalone LSP and existing Evidence Pack v1 verification; normal scans require no external API, license authentication, or telemetry sender.
- Output safety: mask matches and context in supported report formats and interactive previews; reject config-file plain output while retaining explicit CLI unsafe/plain modes.
- Incomplete scans: propagate file, UTF-8, configuration, rule, Git object and limit failures; refuse incomplete baseline creation and preserve existing baselines.
- Safe masking: preserve line endings and permissions, preflight inputs, reject symlinks and existing backups, and recheck contents before replacement. Multiple-file masking is not a single transaction.
- Git staging: honor GIT_INDEX_FILE, stop generated hooks on findings or errors, and preserve existing hook configuration.
- Detection: narrowly suppress unspecified IPv4 and documented non-person name contexts while retaining other person-name, credential and IPv4 findings. False positives and missed detections remain possible.
- Distribution: prepare Linux x86_64 musl and macOS ARM64 archives with checksums, license originals, third-party notices and required sources. macOS remains an ad-hoc-signed trial asset; Developer ID, notarization and downloaded Gatekeeper behavior are unverified. Windows and standalone LSP binaries are excluded.
- 通常CLIの責務・出力秘匿・不完全な検査の終了コード2とbaseline保全を揃え、maskの改行・権限保持と安全な書換え、別インデックスとフック保全、限定的な誤検出修正を含みます。
- 初回スキャン→ステージ済み変更→次回利用の導線と、公開状態・履歴の区別を整備しました。実作業でのrc.4継続利用は未確認で、合成検査の成功とは区別します。

### Publication evidence / 公開結果

The signed tag points to main `61fda63`, whose Ubuntu/macOS, musl, MSRV and guard CI passed. Final archives were rebuilt and verified natively on both OSes, then public downloads were matched to those verified assets. crates.io publication is excluded.

署名タグ・最終ZIP2種・SHA256SUMSを公開し、公開後の取得と一致を確認済みです。[配布元CI](https://github.com/mt4110/veil-rs/actions/runs/38824887889)は公開元の検証記録であり、後続の文書更新のCI結果ではありません。macOSの署名制限・実開発での継続利用未確認は引き続き適用されます。

## Legacy development record labelled 0.22.0 - 2026-02-07 "Guardrails & Evidence / ガードレールと証拠"

Historical development notes below are retained as written. This heading is not evidence of a published 0.22.0 release. 以下は旧開発記録で、0.22.0の公開実績を示しません。

### English

This release solidifies the development lifecycle by introducing deterministic guardrails and mandatory evidence (SOT) for all code changes.

#### 🛡️ Guardrails: Drift Check & SOT Enforcement
- **Drift Check Stabilization**: Unified the detection of CI configuration, documentation, and SOT alignment to prevent configuration drift.
- **I/O & FS Testability**: Refactored `prverify` to use `os.DirFS` and `fstest.MapFS`, making the guardrails themselves fully testable and reproducible.
- **Deterministic SOT Selection**: Implemented strict rules for SOT discovery (`--wanted-pr`, Max PR fallback) to eliminate ambiguity in evidence tracking.
- **SOT Contract**: CI now strictly enforces that any code change must be accompanied by a Source of Truth (SOT) document.

#### 📜 Invariants Kept
- **Merge commit policy**: Avoid squash/rebase rewriting and preserve commit SHAs referenced by evidence.
- **CI as Observability**: CI artifacts in `.local/ci/` are treated as fixed, auditable evidence points.
- **SOT as Contract**: Code changes without SOT are blocked by default.

---

### 日本語

本リリースは、決定論的なガードレールと、すべてのコード変更に対する証拠（SOT）の強制を導入することで、開発ライフサイクルを強固なものにします。

#### 🛡️ ガードレール: Drift Check と SOT 強制
- **Drift Check の安定化**: CI設定、ドキュメント、SOTの整合性検知を統合し、設定の乖離（ドリフト）を未然に防ぎます。
- **I/O & FS のテスト容易性**: `prverify` を `os.DirFS` および `fstest.MapFS` を使用するようにリファクタリングし、ガードレール自体のテストと再現を容易にしました。
- **決定論的な SOT 選択**: SOT探索の厳格なルール（`--wanted-pr`、最大PRへのフォールバック）を実装し、証拠追跡の曖昧さを排除しました。
- **SOT 契約**: コードの変更には必ず Source of Truth (SOT) ドキュメントを伴うことを CI で厳格に強制します。

#### 📜 守られた不変条件
- **マージコミットポリシー**: 証拠の安定性を保証するため、コミット履歴の SHA を維持します。
- **観測点としての CI**: `.local/ci/` 以下の CI 成果物を、固定された監査可能な証拠として扱います。
- **契約としての SOT**: SOT のないコード変更はデフォルトでブロックされます。

## [0.12.0] - 2025-12-19

### 🛡️ Guardian: Operator UX & Resilience
This release completes the "Resilience & Observability" phase, ensuring Guardian is robust against corruption, conflicts, and network issues, while providing clear actionable feedback to operators.

- **Resilience Policies**: Implemented deterministic "Quarantine & Fallback" policies for corrupt or conflicting cache entries.
- **Operator UX**: Error messages now include explicit **remediation hints** (e.g., "Run online to self-heal") when offline recovery fails.
- **Observability**: Fully instrumented `OsvClient` and `ConcurrencyGate` with unified metrics (cache hit/miss, network retries, gate waits, 429s).
- **Cache Contract**: Strict alignment of NormKey hashing (16-char hex) and v1 envelope schema with documentation.
- **AI Workflow**: Formalized AI contribution rules in `docs/ai/`.

## [0.11.3] - 2025-12-17 "Guardian Stability Track"

**Guardian Next stability track completed**: This release solidifies the OSV integration with a focus on reliability, concurrency control, and crash safety.

### 🛡️ Guardian: Robustness & Resilience
- **Concurrency Control**: Implemented `ConcurrencyGate` (`max_in_flight: 8`) and budget-aware permits to prevent OSV API overload.
- **Atomic Writes**: Cache files are now written atomically (tmp + sync + rename) to guarantee no partial JSON corruption on crashes.
- **File Locking**: Added multi-process file locking (`fs2`) to safely handle parallel scans (e.g., CI jobs) sharing the same cache.
- **Key Versioning**: Migrated cache storage to a `v1/` directory with strict filename normalization and collision avoidance, preserving legacy read compatibility.
- **Retry Policy**: Enhanced retry logic with jittered backoff and strict `Retry-After` header respect.

## [0.11.2] - 2025-12-16

### 🛡️ Guardian: Performance Improvements (OSV)
This release focuses on optimizing the OSV client performance and observability within Veil Guardian.

- **Added**: `--debug-metrics` flag to output detailed performance metrics (request counts, cache hits, timings) to stderr.
- **Improved**: Implemented request coalescing for OSV queries and vulnerability details. Concurrent requests for the same resource now result in a single network call, significantly reducing bandwidth and API load.
- **Verified**: Concurrency tests confirm that N parallel requests result in 1 network call, with N-1 coalesced waiters.

## [0.10.0] - 2025-12-13

### 🛡️ Guardian: NPM & OSV Support
- **Package Scanning**: Added support for parsing `package-lock.json` (v1, v2, v3) in `veil guardian check`.
- **OSV Integration**: Now queries the Open Source Vulnerabilities (OSV) API for specialized vulnerability data.
- **Offline Mode**: Added `--offline` flag to scan using cached advisory data without network access.
- **Secure Failures**: Offline cache misses now correctly error instead of silently passing.
- **Improved UX**: Better error tips when lockfiles are missing or invalid.

## [0.9.1] - 2025-12-13

### "Stop the Bleeding"
Veil v0.9.1 introduces **Baseline Scanning**, allowing legacy repositories to adopt secret scanning without being blocked by technical debt.

### Added
- **Baseline Scanning**: New `--baseline` and `--write-baseline` flags to snapshot and suppress existing secrets.
- **Improved Reporting**: HTML report now visualizes "New" vs "Suppressed" findings.
- **Smart Formatter**: JSON/Text outputs focus on actionable new findings, reducing noise.
- **Baseline UX**: Standardized exit codes (`0`: Clean/Suppressed, `1`: New Leak, `2`: Error) and log messages.

### Changed
- **CLI**: `veil scan` logic updated to support 3-state outcomes (No secrets, No new secrets, New secrets).
- **Docs**: Comprehensive guide added at `docs/baseline/usage.md`.
- **Output**: JSON summary now includes: `total_findings`, `new_findings`, `baseline_suppressed` (stable keys).

## [0.8.0] - 2025-12-10 "DX & Delivery"

### 🚀 Highlights
- **Delivery**: New installation methods via `install.sh` (curl | sh) and Nix Flake.
- **CLI UX**: Improved `veil scan --help` and added `veil doctor`.
- **Git History Scan**: Full support for scanning git history (`veil git scan`) and Pull Requests (`--pr`).

### ⚡ Improvements
- **Security**: Added DoS/ReDoS protection limits (line length, file size) and `config check` command.
- **Scanning**: Full support for scanning git history (`veil git scan`), Pull Requests (`--pr`), and staged files (`--staged`).
- **Resilience**: Unified binary/large file skipping logic to prevent CI chokes.

### 🛡️ Security
- **Threat Model**: Documented security boundaries in `docs/security/threat_model.md`.
- **Policy**: `veil.toml` supports `fail_on_score` (default 0) for safe-by-default options.

### 🐛 Fixes
- Fixed exit codes to be CI-friendly.
- Fixed `veil.toml` ignore patterns for test data.
