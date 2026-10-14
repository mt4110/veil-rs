# Threat Model

## Asset
- **User Source Code**: The primary asset `veil-rs` protects is the user's source code and configuration files, helping detect accidental inclusion of secrets (credentials, keys). Detection cannot ensure that no secret is committed.
- **Developer Environment**: `veil-rs` runs in the developer's local environment or CI/CD pipeline. The local scanner should bound resource use and avoid executing scanned content; these are design goals, not a proof against all RCE/DoS inputs.

## Threats & Mitigations

### 1. Denial of Service (DoS) via Malicious Input
**Threat**: An attacker commits a crafted file (e.g. 10GB text, or ReDoS pattern) to the repo, causing `veil scan` to hang or crash CI.
**Mitigation**:
- **Max File Size**: Default limit (1,000,000 bytes) is applied. Oversized text/source files make the scan incomplete (exit 2); binary exclusions are reported separately.
- **Binary Detection**: Files with null bytes in header are skipped.
- **Regex Safety**: `veil config check` detects risky patterns (e.g. nested quantifiers). Rust `regex` crate guarantees linear time execution for searches (automata-based).

### 2. False Negatives (Missed Secrets)
**Threat**: A secret is committed because `veil-rs` failed to detect it.
**Mitigation**:
- **Rule Confidence**: High-severity rules target high-entropy patterns with specific prefixes (e.g. `AKIA...`).
- **Validation**: Review changed rules with generated positive/negative examples. Rule coverage and dogfood results do not establish that every secret pattern is detected.

### 3. False Positives (Disrupted Workflow)
**Threat**: Developers are blocked from committing safe code due to incorrect detection.
**Mitigation**:
- **Allowlisting**: Support for `// veil:ignore` comments and globally via `veil.toml`.
- **Failure policy**: Default configuration has no score failure threshold. `--fail-on-findings 1` fails on any unsuppressed finding; a threshold of 0 is invalid and returns exit 2. Generated init profiles can add score thresholds.
- **Baseline limits**: Matching uses masked text and location, so a different raw value may remain suppressed at the same location. Baseline creation rejects finding/file-count truncation, oversized text/source input, and read/UTF-8 errors before opening its destination, returns exit code 2, and preserves any existing baseline. Ignored and binary files may still be excluded.

### 4. Supply Chain / Remote Execution
**Threat**: `veil-rs` fetches malicious remote rules or executes code.
**Mitigation**:
- **Local OSS profile**: Remote rule fetching is disabled. Legacy remote settings fail with a migration error. Local signed packs require an explicit trust anchor and expiration checks. The preserved legacy Guardian/UI crates have separate network boundaries.
- **Sandboxing**: `veil-rs` does not execute found files; it only reads them.

## Output and baseline metadata

Masking applies to detected portions, not every possible secret or identifier. Paths, Git metadata, and undetected text can remain in output.
In the table formatter, the Match column is `<REDACTED>` unless raw output is explicitly enabled by `--unsafe` or `--mask-mode plain`. Config-file `output.mask_mode = "plain"` is rejected with exit 2, even if a redact flag is also supplied. The published 0.21.0-rc.1 table formatter still prints raw matches even with redact; this source fix does not change that existing ZIP. Until a corrected binary is distributed, use text or JSON with `--mask-mode redact` for sharing.
Review reports before sharing and avoid raw-output options. Baseline suppression does not establish rotation, remediation, or acceptance of risk.
See [usage guidelines](../cli/usage-guidelines.md) and [baseline matching](../baseline/usage.md).
