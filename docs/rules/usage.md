# Rules Management

`veil-rs` provides transparency into its active rule set, allowing you to see exactly what is being detected and why.

## Built-in & Remote Rules

## Effective Rule Set

Veil combines rules from three sources into a single "Effective Rule Set". The merge order is **Built-in → Remote → Local**.

1.  **Built-in Rules**: Hardcoded core rules (AWS, GitHub, Slack, etc.).
2.  **Remote Rules**: Fetched from `VEIL_REMOTE_RULES_URL` (if configured).
3.  **Local Overrides**: Custom rules defined in `veil.toml`.

Local rules in `veil.toml` can override both built-in and remote rules.

## Listing Rules

To see all currently active rules:

```bash
veil rules list
```

This commands prints a table with:
- **ID**: The unique identifier (e.g., `creds.aws.access_key_id`).
- **Severity**: Default severity level.
- **Score**: Default score.
- **Category**: Classification (secret, pii, etc.).
- **Description**: Human-readable summary.

### Filtering by Severity

You can filter the list to show only rules at or above a certain severity:

```bash
# Show only HIGH and CRITICAL rules
veil rules list --severity HIGH
```

## Explaining a Rule

To view detailed information about a specific rule, including its regex pattern and tags:

```bash
veil rules explain creds.aws.access_key_id
```

Output example:

```text
ID:          creds.aws.access_key_id
Description: AWS Access Key ID
Severity:    HIGH
Score:       85
Category:    secret
Tags:        credential, cloud, aws, critical

Pattern:
\b(AKIA|ASIA|AGPA|AIDA|AROA|AIPA|ANPA|ANVA)[0-9A-Z]{16}\b

Context:
Before: 1 lines / After: 1 lines
```

If the rule ID does not exist in the effective rule set, `veil rules explain` exits with an error.

## Custom Rules

When you define a custom rule in `veil.toml`, it immediately appears in `veil rules list`.

**Example `veil.toml`**:
```toml
[rules.internal_project_id]
enabled = true
description = "Internal Project ID"
pattern = "PROJ-\\d{4}"
severity = "medium"
```

Running `veil rules list` will now include `internal_project_id`.

**Note**: Rules with `enabled = false` are removed from the effective set and will not appear in `veil rules list`.
