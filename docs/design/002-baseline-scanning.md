# v0.9.x Design: Baseline & Incremental Scanning

**Status**: Draft
**Target Version**: v0.9.1
**Epic**: B (Baseline)

## Objective
Enable `veil` to be introduced into legacy codebases with existing secrets ("Debt") without breaking the build, while ensuring *new* secrets are strictly blocked.

## Concept: snapshot-based exclusion
Instead of complex "since git commit" logic (which is flaky in shallow clones), we use a **Baseline File**.
This file records a snapshot of all *currently known* findings. Future scans compare findings against this snapshot.

## Baseline File Format (`veil.baseline.json`)

```json
{
  "schema_version": "1.0",
  "generated_at": "2025-12-11T12:00:00Z",
  "findings": [
    {
      "rule_id": "creds.aws.access_key",
      "file_path": "legacy/data.sql",
      "line": 120,
      "fingerprint": "sha256(rule_id + file_path + masked_snippet)" // TBD: Include surrounding context?"
    }
  ]
}
```

### Fingerprinting
Crucial for stability. If code moves lines (refactor), we want to recognize the finding is "the same".
*   **Strict**: File + Line + Content. (Breaks on line shifts).
*   **Resilient**: File + AST/Context? (Too complex for v0.9).
*   **Choice**: `hash(file_path + rule_id + snippet)`.
    *   If line moves, snippet stays same -> matched.
    *   If snippet changes -> likely a *new* secret or modified secret -> Report it.

## CLI Workflow

1.  **Generate Baseline**
    ```bash
    veil scan . --format json --write-baseline veil.baseline.json
    ```
    *   This records all 1500 existing issues.

2.  **Scan with Baseline**
    ```bash
    veil scan . --baseline veil.baseline.json
    ```
    *   Loads baseline.
    *   Scans project -> 1502 findings.
    *   Matches 1500 against baseline.
    *   **Reports**: 2 New Findings.
    *   **Exit Code**: Non-zero (because 2 new issues).

## Reporting Integration
*   HTML Report should separate "New" (Active) vs "Baseline" (Suppressed).
    *   Summary Card: "2 New Findings (1500 Suppressed)".
    *   Table: Toggle to show/hide suppressed.

## Implementation Tasks (v0.9.1)
1.  Define `Baseline` struct and serialization.
2.  Implement `Fingerprint` generation logic.
3.  Update `Scanner` output logic to filter against loaded Baseline.
4.  Updates to CLI args (`--baseline`, `--write-baseline`).
