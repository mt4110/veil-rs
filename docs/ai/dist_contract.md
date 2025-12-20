# Phase 8 Distribution Contract (v0.14.x)

**Purpose**: Define the immutable contract for release artifacts to ensure "zero hesitation" automation.

## 1. Directory Structure

All artifacts MUST be generated in:
`dist/publish/<VERSION>/`

## 2. Required Artifacts (The "Big 4")

Every release generation MUST produce exactly these 4 files:

1.  `PUBLISH_<VERSION>.md` (The Release PR body)
2.  `RELEASE_BODY_<VERSION>.md` (The GitHub Release text)
3.  `X_<VERSION>.md` (Social media announcement draft)
4.  `AI_PACK_<VERSION>.txt` (Context pack for LLM/Review)

**Constraint**: `AI_PACK` MUST have the extension `.txt`.
**Reason**: To strictly differentiate it from "content to be published" (markdown).

## 3. CI/Local Parity & Safety

- **Local**: Generates all 4 artifacts.
- **CI**: Verifies integrity.
    - CI `actions/upload-artifact` MUST be configured to upload `**/*.md` ONLY.
    - usage of `AI_PACK` in CI artifacts is **FORBIDDEN**.
    - `check.sh` MUST fail if `AI_PACK` overlaps with publishable extensions.
- **Environment**: Scripts must run on macOS (bash 3.x) and Linux (bash 4.x+). No `mapfile`.

## 4. Exit Codes (Unified)

- `2`: Usage error (wrong arguments).
- `3`: Guardrails/Check failure (validation).
- `4`: Generation failure (runtime).
- `5`: I/O error (artifacts missing/unwritable).
