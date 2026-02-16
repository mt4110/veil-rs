# PR Source of Truth (SOT)

This directory contains Source of Truth (SOT) documents for each Pull Request.
A SOT document is the persistent record of the PR's intent, changes, and verification evidence.

## SOT File Naming

Recommended (before PR number is known):

- `docs/pr/PR-TBD-<release>-epic-<epic>[-<slug>].md`
  - e.g. `docs/pr/PR-TBD-v0.19.0-epic-a.md`
  - e.g. `docs/pr/PR-TBD-v0.19.0-epic-b-sot-ux.md`

Optional (after PR number is assigned):

- rename to `docs/pr/PR-<pr>-<release>-epic-<epic>[-<slug>].md`
  - e.g. `docs/pr/PR-123-v0.19.0-epic-b-sot-ux.md`

The key is that the file exists *before* you open the PR, so the PR body can link to it reliably.

## Workflow (Manual)

1. Create a new SOT file:
   - `docs/pr/PR-TBD-<release>-epic-<epic>[-<slug>].md`
   - e.g. `cp docs/pr/sot_template.md docs/pr/PR-TBD-...`
2. Fill standard sections:
   - SOT / What / Verification / Evidence
3. Copy-paste the SOT filename into your PR description:
   ```md
   ### SOT
   - docs/pr/PR-TBD-...
   ```
4. Keep the SOT updated as the PR evolves.

## Why SOT?

- Consistent naming + metadata via manual creation.
- Persistent record that survives squash merges.
- Single place for complex verification logs and evidence.

> [!NOTE]
> **Manual SOT Creation**
> Since `veil sot new` is removed, please use `cp docs/pr/sot_template.md ...` or create the file manually.
