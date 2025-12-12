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


## [v0.8.0] - 2025-12-10 "DX & Delivery"

### 🚀 Highlights
- **Delivery**: New installation methods via `install.sh` (curl | sh) and Nix Flake.
- **CLI UX**: Improved `veil scan --help` and added `veil doctor`.
- **Git History Scan**: Full support for scanning git history (`veil git scan`) and Pull Requests (`--pr`).

### ⚡ Improvements
- **Security**: Added DoS/ReDoS protection limits (line length, file size).
- **UX**: Refactored `veil scan` help messages for clarity.
- **Safety**: Added `config check` to validate `veil.toml` regex safety.

### 🛡️ Security
- **Threat Model**: Documented security boundaries in `docs/security/threat_model.md`.


### 🚀 New Features
- **`veil config check` command**: Validates configuration for potential ReDoS patterns in regexes and ensures config correctness. Checks both local `veil.toml` and remote rules.
- **Unified Scanning Pipeline**: Refactored scanning logic (`scan_data`) ensures consistent behavior across all scan modes (`scan .`, `--staged`, `<commit>`, `--since`).
- **Binary & Large File Handling**: 
  - Unified detection logic in core.
  - Automatically skips binary files (via null byte check).
  - Skips files larger than `max_file_size` (default 1MB) to prevent DoS.

### 🛡️ Security & Performance
- **DoS Resilience**: Verified safety against 5MB+ inputs and 10k+ matches.
- **ReDoS Prevention**: Config check now warns about dangerous nested quantifiers (e.g. `(.+)+`).
- **Threat Model**: Added `docs/security/threat_model.md` covering mitigation strategies.

### ⚙️ Configuration & Policy
- **Refactored Config Loader**: Centralized logic for loading Org (`VEIL_ORG_RULES`), Project (`veil.toml`), and CLI overrides.
- **Fail-on-Score Default**: Explicitly set to `0` (non-blocking) in `veil.toml` to support "Safe by Default" CI adoption.
- **Test Data Policy**: `docs/TESTING_SECRETS.md` created to document "Runtime Generation" policy for secrets in tests.

### 🐛 Fixes
- Fixed exit codes to be CI-friendly (defaults to 0 unless configured otherwise).
- Fixed `veil.toml` ignore patterns to correctly handle test data folders.
- Fixed duplicate list items in README and CI examples.
