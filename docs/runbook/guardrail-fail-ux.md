# Guardrail Failure UX Guidelines

To ensure a smooth developer experience, `prverify` failures must be **Deterministic** and **Actionable within 1 Scroll**.

## 1. Enforcement Timeline & Output
`prverify` output format depends on the version/PR:

- **PR41 (Current Main)**: Outputs a **Header** + `Cause/Action/Fix`. `NO_COLOR` removes ANSI but formatting remains.
- **PR42+ (Registry v1)**: Outputs `Reason/Fix/Next` (No Header). `NO_COLOR` switches to plain text (no ANSI symbols).

## 2. Output Structure

### PR41 (Current Main Output)
Used by legacy drift checks until PR42 merge.

```text
[<Category> Drift] <Summary>
  Cause:  <reason>
  Action: <guidance>
  Fix:    <command>
```
*Note: PR41 output always includes a category header.*

### PR42+ (New Output)
Standard for Registry v1 and future checks.

#### Text / NO_COLOR
```text
Reason: <reason>
Fix:    <command>
Next:   <command>
```

#### ANSI (Color)
ANSI is enabled when `NO_COLOR` is unset.
- **Reason**: Bold
- **Fix**: Green command
- **Next**: Blue command

### Example (PR42+)
```text
Reason: expires_at (2025-01-01) is in the past
Fix:    edit ops/exceptions.toml
Next:   nix run .#prverify
```

> [!NOTE]
> **Header Policy**: PR41 outputs a header; PR42+ does **not** output a header (category is internal to existing error struct).

## 2. Determinism Rules
- **Ordering**: Output items must be **sorted deterministically** (e.g., by ID asc, then Path asc, then Rule asc).
- **Formatting**:
    - No absolute paths (use relative to repo root).
    - Stable newlines and indentation.
    - No timestamps in output (unless strictly necessary and stable).
- **Environment Independence**: Output should look the same on local dev (Mac/Linux) and CI.

## 3. Pagination / Limiting
- **Max Items**: Display a maximum of **10** items per category to preserve scroll context.
- **Overflow**: If more than 10 items exist, show `... and N more` at the bottom.

## 4. Color & Styling
- **Default (Recommendation)**: Output must be fully readable with **NO color**.
- **Optional**: If colors are used for emphasis (e.g., Red for errors, Green for fixes):
    - Must respect `NO_COLOR` env var (disable if set).
    - Color codes must never convey meaning alone (text content must differ).
