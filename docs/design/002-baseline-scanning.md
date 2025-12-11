# Baseline Scanning (v0.9.1)

Baseline scanning allows users to "grandfather in" existing findings in a legacy codebase, ensuring that the scan only fails on **new** findings.

## Concept

1.  **Snapshot**: A user generates a `veil.baseline.json` file containing all current findings.
2.  **Filter**: During a scan, Veil compares current findings against the baseline.
3.  **Status**:
    *   **New**: Not in baseline (Exit code 1).
    *   **Suppressed**: Matches baseline (Exit code 0, reported as suppressed).

## Data Model

### JSON Schema (`veil.baseline.v1`)

```json
{
  "schema": "veil.baseline.v1",
  "generated_at": "2025-12-10T12:00:00Z",
  "tool": "veil-rs v0.9.1",
  "entries": [
    {
      "fingerprint": "a1b2c3d4...",
      "rule_id": "creds.aws.access_key",
      "path": "src/config.py",
      "line": 42,
      "severity": "High"
    }
  ]
}
```

### Fingerprint Strategy (v1)

We trade robustness for simplicity in v1. A finding matches if the fingerprint matches.

```rust
fingerprint = SHA256(
    format!("{}|{}|{}|{}", rule_id, path, line, masked_snippet)
)
```

**Implications**:
- **Line Numbers**: If code is inserted above, the line number changes, and the baseline breaks (re-baselining required).
- **Refactoring**: File moves break baseline.
- **Security**: The masked snippet is included to prevent collision, but sensitive data is not stored in plain text (though hashed).

## CLI Workflow

### 1. Initialize Baseline
```bash
veil scan --write-baseline veil.baseline.json
```
- Runs a full scan.
- Writes all findings to `veil.baseline.json`.
- Exits with 0 (Success).

### 2. CI / Daily Scan
```bash
veil scan --baseline veil.baseline.json
```
- Runs scan.
- Calculates fingerprints for all findings.
- **Suppressed**: Findings present in `veil.baseline.json`.
- **New**: Findings NOT present.
- **Fail Condition**: If `# New > 0`, exit code 1.
