# veil-rs Roadmap

## ✅ v0.5.0: Core Engine & Safety (Current Release)
The foundation of a "Safe & Fast Static Secret Scanner".
- **Rule Engine 2.0**: Context-aware scanning (look-around lines), Scoring system, and Tagging.
- **RED Rules**: High-confidence built-in rules (AWS, GitHub, Slack, etc.) to minimize false positives.
- **Safe Output (Epic C)**: `Redact` / `Partial` masking by default. `plain` output restricted via CLI flags.
- **Performance & Scale (Epic D)**: Global scanning limits (`max_findings`) and benchmarking infrastructure (`divan`).

## 🚀 Upcoming

### v0.6.0: Integration & Customization

#### Epic E: Secret Registry & Custom Rules
A FastAPI-like local registry for managing secret patterns.
- **Local Secret Registry**: HTTP API (`/secrets`) to register custom logs/secrets locally.
- **Dynamic Rule Loading**: Convert registered secrets into veil-core rules on the fly.
- **Custom Config**: Support for `[custom_rules]` in `veil.toml` and external rule sources.

#### Epic F: Git & Dev Flow Integration
Seamless integration into the developer workflow, matching functionality of tools like gitleaks.
- **Git Aware**: `veil scan --staged` (pre-commit support) and `veil scan --history`.
- **CI/CD Templates**: Official GitHub Actions and GitLab CI configurations.
- **Pre-commit Hooks**: Ready-to-use `.pre-commit-config.yaml`.

### v0.7.0+: Runtime & Enterprise features

#### Epic G: Runtime Masking & Logging
Extending veil's protection beyond static code analysis into runtime logs.
- **`veil-logger` Crate**: Middleware/Layer for `tracing` and `log` crates.
- **Runtime Redaction**: Apply `veil-core` rules to application logs in real-time.
- **DLP Integration**: Unified rule set for both source code scanning and runtime log auditing.
