# Threat Model

## Asset
- **User Source Code**: The primary asset `veil-rs` protects is the user's source code and configuration files, specifically ensuring no secrets (credentials, keys) are committed.
- **Developer Environment**: `veil-rs` runs in the developer's local environment or CI/CD pipeline. Requires "safe" execution (no RCE/DoS).

## Threats & Mitigations

### 1. Denial of Service (DoS) via Malicious Input
**Threat**: An attacker commits a crafted file (e.g. 10GB text, or ReDoS pattern) to the repo, causing `veil scan` to hang or crash CI.
**Mitigation**:
- **Max File Size**: Default limit (1MB) strictly enforced. Files larger than this are skipped.
- **Binary Detection**: Files with null bytes in header are skipped.
- **Regex Safety**: `veil config check` detects risky patterns (e.g. nested quantifiers). Rust `regex` crate guarantees linear time execution for searches (automata-based).

### 2. False Negatives (Missed Secrets)
**Threat**: A secret is committed because `veil-rs` failed to detect it.
**Mitigation**:
- **Rule Confidence**: High-severity rules target high-entropy patterns with specific prefixes (e.g. `AKIA...`).
- **Validation**: Some rules include checksum validation (e.g. Lush check).
- **Dogfooding**: Continuous scanning of open-source datasets to tune rules.

### 3. False Positives (Disrupted Workflow)
**Threat**: Developers are blocked from committing safe code due to incorrect detection.
**Mitigation**:
- **Allowlisting**: Support for `// veil:ignore` comments and globally via `veil.toml`.
- **Baseline**: `fail_on_score = 0` by default ensures non-blocking behavior unless explicitly configured.

### 4. Supply Chain / Remote Execution
**Threat**: `veil-rs` fetches malicious remote rules or executes code.
**Mitigation**:
- **Remote Rules**: Fetched via HTTPS. Parsed as JSON. No executable code downloaded (only data).
- **Sandboxing**: `veil-rs` does not execute found files; it only reads them.
