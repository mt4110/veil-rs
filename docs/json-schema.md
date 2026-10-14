# veil-v1 JSON output contract

Status: `reference`

`veil scan --format json` writes a single JSON object to stdout. Diagnostics use stderr.

```json
{
  "schemaVersion": "veil-v1",
  "summary": {
    "total_files": 1,
    "scanned_files": 1,
    "skipped_files": 0,
    "total_findings": 0,
    "new_findings": 0,
    "baseline_suppressed": 0,
    "limit_reached": false,
    "file_limit_reached": false,
    "max_file_size_reached": false,
    "read_error_reached": false,
    "duration_ms": 0,
    "baseline_path": null,
    "severity_counts": {},
    "builtin_skips": []
  },
  "findings": []
}
```

`total_findings` includes baseline-suppressed findings; `new_findings` is the unsuppressed count.
`findings` contains unsuppressed findings only. `baseline_suppressed` reports historical suppression, not an approval.
`severity_counts` counts unsuppressed findings, using `Low`, `Medium`, `High`, `Critical` keys.
`builtin_skips` reports built-in ignored directory names. `baseline_path` is a string or null.
File counters are numbers; `duration_ms` is measured elapsed milliseconds. Field order is not a contract.

`limit_reached`, `file_limit_reached`, `max_file_size_reached` and `read_error_reached` indicate incomplete results and cause CLI exit code 2. Incomplete scans cannot create or replace a baseline.
Exit code 0 does not prove that every file was scanned: ignored, binary, or oversized files can be skipped.

## Finding fields and sensitive data

Findings retain the existing field names: `path`, `line_number`, `rule_id`, `severity`, `score`, `grade`, `masked_snippet`, `line_content`, `matched_content`, `context_before`, `context_after`.
Optional Git metadata is `commit_sha`, `author`, `date`.

Default masked output uses `masked_snippet` for `line_content`, `<REDACTED>` for `matched_content`, and replaces each context line with `<REDACTED>`.
Context length is preserved, but neighboring raw text is not emitted. Rule-specific or configured masking controls `masked_snippet`.
Partial masking completely hides secrets of eight characters or fewer, preventing overlapping head/tail fragments from disclosing the full value. Longer values show only the last four characters; the leading characters remain hidden.

Explicit CLI `--unsafe` or `--mask-mode plain` permits raw output. Config-file `output.mask_mode = "plain"` is forbidden and causes exit code 2, even with a CLI redact override. Do not save or share this output without reviewing its contents.

This corrects the previous raw-field disclosure while retaining field names and types. Consumers needing raw material must opt in deliberately.
Internal `veil-core::Finding` retains raw values; the CLI applies this masking contract when serializing shared scan output.
