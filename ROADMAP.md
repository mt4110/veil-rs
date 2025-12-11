# Veil-rs Roadmap (v0.5 → v1.0)

This document tracks the evolution of veil-rs from the first "safe core" release (v0.5.0) to a stable v1.0.

## Phase 1 – Hardening (v0.5.x)
**Goal**: Validate the core engine, masking, and JSON output under real-world usage.

### v0.5.1 – Masking & Summary Fixes
- **Masking Config**: Wire `MaskingConfig.placeholder` from `veil-config` into `veil-core::masking::apply_masks`.
    - Keep `<REDACTED>` as default.
    - Allow customization via config.
- **Summary Improvements**:
    - Track `total_files`, `scanned_files`, `skipped_files` (binary/ignore/limit).
    - Track `limit_reached` status.
    - Expose improved summary in JSON output.

### v0.5.2 – Output Contract & JSON Schema
- **JSON Contract**: Define stable fields, types, and semantics for Findings and Summary.
- **Documentation**: Add `docs/json-schema.md` or `schema/veil-finding-v1.json`.
- **Testing**: Add "Golden File" tests (compare `scan --format json` output against expected).

### v0.5.3 – Dogfooding Round 1
- **Real-world Scans**: Run against `veil-rs`, `rec-watch`, `crossport`.
- **Tuning**: Collect FP/FNs, add fixtures, and tune rules/scores.

### v0.5.4 – veri-rs Integration
- **Schema Versioning**: Add `schemaVersion` (e.g. `veil-v1`) to JSON output.
- **Verification**: Document how external tools should consume the output.

### v0.5.5 – Load & Performance (Optional)
- **Scale Test**: 100MB+ repos/logs.
- **Metrics**: Execution time, RSS memory usage.

---

## Phase 2 – Integration (v0.6.x)
**Goal**: Make veil-rs easy to consume by machines (CI/CD, external tools) and developers (Git integration).

### v0.6.0 – Integration & Schema (Session 7)
- **Schema Versioning**: Add `schemaVersion` to JSON output.
- **Contract**: Define stable API for integrations (veri-rs).
- **Docs**: Add Integration Guide to README.

### v0.6.1 – Git & CI Integration (Session 8)
- **Pre-commit**: Add support and templates.
- **GitHub Actions**: Official workflow snippets.
- **Git Hooks**: Native hook examples.

### v0.6.2 – Multi-Repo Dogfooding (Session 9)
- **Validation**: Scan external OSS repos to tune rules.
- **False Positives**: Refine rules based on broader data.

---

## Phase 3 – Security Hardening (v0.7.x)
**Goal**: Transform veil-rs into a "security-first" hardened tool.

### v0.7.0 – Hardening & Best Practices (Sessions 10-12)
- **Threat Model**: Document security boundaries and guarantees.
- **Config Safety**: Validate config and block dangerous regexes.
- **DoS Resistance**: Protection against massive repositories/binaries.
- **Best Practices**: Recommended configurations for different scales.

---

## Phase 4 – DX & Delivery (v0.8.x)
**Goal**: Make the tool "installable and usable by anyone in 5 minutes".
Target audience: Individual developers and small teams.

### v0.8.0 – Delivery & CLI UX (Session 13-15)
- **Delivery**: `install.sh`, Nix Flake support, `README` quick start.
- **CLI UX**: Refactored `veil scan --help`, added `veil doctor`.
- **First Impression**: Ensure the tool feels "premium" and "easy" from the first run.

### v0.8.x – Rule & Report DX (Planned)
- **HTML Report**: Improve UX with filtering, search, and summary charts.
- **Rules DX**: `veil rules list` / `explain`.
- **Wizard**: Enhanced `veil init` wizard for CI/Test data configs.

---

## Phase 5 – Teams & Policy (v0.9.x)
**Goal**: Features for organizational scaling and policy enforcement.
Target audience: Security teams, organizations, enterprise usage.

### Epic P: Policy / Org Features
- **Org Config**: Documentation and examples for `VEIL_ORG_RULES`.
- **Policy Layering**: Explicit priority rules between Org vs Project config.

### Epic CI: CI / Baseline Operations
- **Baseline**: "New findings only" mode (`--baseline`).
- **CI Patterns**: Advanced GitHub Actions / pre-commit patterns for teams.

### Epic R: Reporting for Teams
- **Actionable HTML**: "Fix this first" sections in reports.
- **Metrics**: Aggregated metrics for team dashboards.

### Epic S: Stability Declaration
- **Versioning**: Explicit versioning policy for CLI, Config, and JSON.
- **Breaking Changes**: Documentation on how breaking changes are handled.

---

## Phase 6 – Stable v1.0.0
**Goal**: A stable, reliable, and finished OSS tool.

### Definition of Done for v1.0.0
1.  **Stable Specs**:
    - JSON Schema (`veil-v1`) is frozen.
    - CLI flags and behavior are stable.
    - `veil.toml` structure is forward-compatible.
2.  **Safety & Performance**:
    - DoS / ReDoS protections are verified.
    - Proven stability on large repositories.
3.  **DX & Documentation**:
    - Installation is trivial (Install script / Nix / Cargo).
    - `doctor` provides useful troubleshooting.
    - Complete documentation for Integrations (CI, Git).
4.  **Team Readiness**:
    - Ready for Org-wide deployment.
