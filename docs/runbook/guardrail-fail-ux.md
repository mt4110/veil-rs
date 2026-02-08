# Guardrail Failure UX Guidelines

To ensure a smooth developer experience, `prverify` failures must be **Deterministic** and **Actionable within 1 Scroll**.

## 1. Output Structure
Failure blocks must match `driftError.Print()` output. Format adapts to `NO_COLOR`.

### Default (Text / NO_COLOR)
```text
Reason: <reason>
Fix:    <command>
Next:   <command>
```

### ANSI (Color)
ANSI is enabled when `NO_COLOR` is unset. Set `NO_COLOR` for plain logs / CI determinism.
- **Reason**: Bold
- **Fix**: Green command
- **Next**: Blue command

### Example
```text
Reason: expires_at (2025-01-01) is in the past
Fix:    edit ops/exceptions.toml
Next:   nix run .#prverify
```

> [!NOTE]
> `driftError.Print` does not currently output a header (e.g. `[Registry Drift]`). The category is internal to the error struct.

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
