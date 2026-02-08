# Guardrail Failure UX Guidelines

To ensure a smooth developer experience, `prverify` failures must be **Deterministic** and **Actionable within 1 Scroll**.

## 1. Output Structure
Every failure block must follow this exact template:

```text
[<Category> Drift] <Summary>
  Cause:  <Specific reason for failure>
  Action: <High-level guidance>
  Fix:    <Specific command to run>
```

### Example
```text
[Registry Drift] Exception EX-20260208-001 is expired
  Cause:  expires_at (2025-01-01) is in the past
  Action: Remove the exception or extend validity with justification
  Fix:    edit ops/exceptions.toml
```

## 2. Determinism Rules
- **Ordering**: Output items must be sorted securely (e.g., by ID, Path, then Rule).
- **Formatting**:
    - No absolute paths (use relative to repo root).
    - Stable newlines and indentation.
    - No timestamps in output (unless strictly necessary and stable).
- **Environment Independence**: Output should look the same on local dev (Mac/Linux) and CI.

## 3. Pagination / Limiting
- **Max Items**: Display a maximum of **10** items per category to preserve scroll context.
- **Overflow**: If more than 10 items exist, show `... and N more` at the bottom.

## 4. Color & Styling
- **Heading**: Bold/Red for immediate attention.
- **Keys**: Bold (Cause/Action/Fix).
- **Fix Command**: Green (copy-pasteable).
- Use `NO_COLOR` env var to disable coloring if set.
