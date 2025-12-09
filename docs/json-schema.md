# veil-rs JSON Output Contract (Draft – v0.5.x)

This document describes the JSON output format produced by `veil scan --format json`.
The goal is to provide a stable contract for tools like `veri-rs` and CI systems.

**Status**: v0.5.x draft. Field names and types are expected to be stable from v0.5.1 onward.

## 1. Top-level Structure

A typical JSON output looks like this:

```json
{
  "summary": { /* Summary */ },
  "findings": [ /* Finding[] */ ]
}
```

*   `summary`: Aggregated statistics for the scan.
*   `findings`: List of individual findings.

If no findings are discovered, `findings` will be an empty array, but `summary` is still present.

## 2. Summary Object

```json
{
  "total_files": 10,
  "scanned_files": 8,
  "skipped_files": 2,
  "findings_count": 12,
  "shown_findings": 12,
  "limit_reached": false,
  "duration_ms": 1234,
  "severity_counts": {
    "Critical": 1,
    "High": 3,
    "Medium": 5,
    "Low": 3
  }
}
```

### Fields

*   **`total_files`** (number, required)
    *   Total number of files considered by the scan.
    *   For `scan_path` this is the number of filesystem entries scanned.
    *   For streaming scans (like `veil scan` over git history), this is approximated as `scanned_files + skipped_files`.

*   **`scanned_files`** (number, required)
    *   Number of files that were actually opened and scanned.

*   **`skipped_files`** (number, required)
    *   Number of files that were skipped.
    *   Typical reasons:
        *   Binary files (rule id `BINARY_FILE`).
        *   Files larger than configured max size (rule id `MAX_FILE_SIZE`).

*   **`findings_count`** (number, required)
    *   Total number of findings discovered (including those that might not be shown due to later filters).

*   **`shown_findings`** (number, required)
    *   Number of findings actually included in the `findings` array in this output.

*   **`limit_reached`** (boolean, required)
    *   Indicates whether the scan limit was reached:
        *   `false`: All files were processed within the configured limit.
        *   `true`: The scan stopped early because the maximum number of findings was reached.

*   **`duration_ms`** (number, required)
    *   Duration of the scan in milliseconds.

*   **`severity_counts`** (object, required)
    *   Map from severity level to the number of findings with that severity.
    *   Keys are strings, values are numbers. Known keys include:
        *   `"Critical"`
        *   `"High"`
        *   `"Medium"`
        *   `"Low"`
    *   Other severities may be added in future versions, so consumers should not assume this set is closed.

## 3. Finding Object

A `Finding` describes a single occurrence of a rule match.

Example (fields may be abbreviated here):

```json
{
  "rule_id": "AWS_ACCESS_KEY_ID",
  "severity": "High",
  "path": "src/main.rs",
  "line_number": 42,
  "matched_content": "AKIA....",
  "masked_snippet": "AKIA<REDACTED>",
  "line_content": "let key = \"AKIA<REDACTED>\";",
  "score": 80,
  "grade": "Critical",
  "context_before": ["fn main() {", "  // TODO: remove this"],
  "context_after": ["}", ""]
}
```

### Required Fields

*   **`rule_id`** (string)
    *   Identifier for the rule that triggered this finding.
    *   Example: `"AWS_ACCESS_KEY_ID"`, `"GITHUB_TOKEN"`.

*   **`severity`** (string)
    *   Severity level for this finding.
    *   Expected values include `"Critical"`, `"High"`, `"Medium"`, `"Low"` (Note: serialization preserves case from `Severity` enum, e.g. `"High"`).

*   **`path`** (string)
    *   Path to the file where the finding was discovered (relative to scan root).

*   **`line_number`** (number)
    *   1-based line number of the primary match.

*   **`masked_snippet`** (string)
    *   Masked representation of the matched content, safe for display in logs and CI.
    *   This respects the masking configuration (e.g. `<REDACTED>` or `****`).

*   **`line_content`** (string)
    *   The raw content of the line where the match occurred (potentially containing the secret if not masked, but currently veil-core stores raw line. Be careful with display). 
    *   *Note: In future versions, this might be sanitized by default also.*

### Internal / Optional Fields

*   **`matched_content`** (string, optional)
    *   Raw matched content (the secret itself).
    *   **This field is intended for internal processing only.**

*   **`score`** (number)
    *   Confidence score (0-100).

*   **`grade`** (string)
    *   Confidence grade (e.g. `"Critical"`, `"Safe"`).

*   **`context_before`** (string[], optional)
    *   Lines before the match, used for context.

*   **`context_after`** (string[], optional)
    *   Lines after the match, used for context.

## 4. Schema Versioning

To help external tools validate and evolve with the format, `veil-rs` may include a schema version:

```json
{
  "schemaVersion": "veil-v1",
  "summary": { ... },
  "findings": [ ... ]
}
```

*   **`schemaVersion`** (string, optional in v0.5.x, recommended from v0.5.2 onward)
    *   Example: `"veil-v1"`
    *   Consumers like `veri-rs` can use this to select the appropriate validation logic.
    *   If `schemaVersion` is absent, consumers should assume the current default version (v0.5.x contract).

## 5. Backward Compatibility Notes

*   The field previously known as `truncated` has been renamed to `limit_reached` in v0.5.1.
*   Special “pseudo-findings” such as `BINARY_FILE` and `MAX_FILE_SIZE`:
    *   Are **not** returned in the `findings` array anymore.
    *   Instead, they contribute to **`skipped_files`** in the summary.
*   External tools should rely on:
    *   `summary.limit_reached` to detect truncated scans.
    *   `summary.skipped_files` to understand how many files were skipped.
