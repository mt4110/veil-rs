# PR Source of Truth (SOT)

This directory contains Source of Truth (SOT) documents for each Pull Request.
A SOT document is the persistent record of the PR's intent, changes, and verification evidence.

## SOT File Naming

Recommended (before PR number is known):

- `docs/pr/PR-TBD-<short>.md`  (e.g. `PR-TBD-pr-template-sot.md`)

Optional (after PR number is assigned):

- rename to `docs/pr/PR-1234-<short>.md`

The key is that the file exists *before* you open the PR, so the PR body can link to it reliably.

## Workflow (v0.19.0+)

1. Create a new SOT file:
   ```bash
   veil sot new --epic <A|B|...> --slug <short-name>
   ```
2. Copy the path from the output and paste it into your PR description:
   `SOT: docs/pr/PR-TBD-<short>.md`
3. Keep the SOT updated as the PR evolves.
4. (Optional) rename once the PR number exists.

## Why SOT?

- Directly editable via `cat > ...` commands in the PR template (avoids `$EDITOR` instability).
- Persistent record that survives squash merges.
- Single place for complex verification logs and evidence.
