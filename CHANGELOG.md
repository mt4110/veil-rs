# Changelog

All notable changes to this project will be documented in this file.

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
