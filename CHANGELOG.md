# Changelog

All notable changes to this project will be documented in this file.

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

- **Configs**: JSON summary now includes: `total_findings`, `new_findings`, `baseline_suppressed` (stable keys).

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
