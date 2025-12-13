# v0.9.x Design: Team Reporting & Metrics

**Status**: Draft
**Target Version**: v0.9.1+
**Epic**: R (Reporting)

## Objective
Provide machine-readable, aggregated metrics to support SRE/Security teams in tracking the "Health" of repositories over time.

## Strategy
Do not build a dashboard *inside* the CLI. Instead, provide a standardized **Metric Output** that can be ingested by any dashboard (Grafana, Datadog, Custom App).

## New Output Format: `summary`

```bash
veil scan . --format summary
```

### JSON Schema (`veil.summary.v1`)

```json
{
  "schema": "veil.summary.v1",
  "generated_at": "2025-12-11T12:00:00Z",
  "repo": {
    "git_url": "https://github.com/org/repo.git",
    "branch": "main",
    "commit": "abc1234"
  },
  "metrics": {
    "duration_ms": 1200,
    "files_scanned": 450,
    "total_findings": 12,
    "by_severity": {
      "CRITICAL": 0,
      "HIGH": 2,
      "MEDIUM": 5,
      "LOW": 5
    },
    "top_rules": [
      { "id": "creds.aws", "count": 2 }
    ]
  },
  "policy": {
    "fail_on_severity": "High",
    "baseline_used": true
  }
}
```

## Usage Patterns

### CI/CD Pipe
```yaml
steps:
  - name: Veil Metrics
    run: |
      veil scan . --format summary > metrics.json
      curl -X POST https://metrics.internal/ingest -d @metrics.json
```

## Logic
*   This output should verify consistency with HTML Report Summary Cards.
*   Use shared logic (crate-level struct) for calculating stats to ensure HTML and Summary JSON always match.

## Future
*   Can be extended to support Prometheus format (`--format prometheus`) if requested (e.g. for exporters).
