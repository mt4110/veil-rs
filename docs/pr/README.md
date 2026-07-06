# PR Source of Truth (SOT)

This directory contains Source of Truth (SOT) documents for each Pull Request.
A SOT document is the persistent record of the PR's intent, changes, and verification evidence.

## SOT File Naming

Recommended (before PR number is known):

- `docs/pr/PR-TBD-<slug>.md`
  - e.g. `docs/pr/PR-TBD-sot-template-helper.md`
- `docs/pr/PR-TBD-<release>-epic-<epic>[-<slug>].md`
  - e.g. `docs/pr/PR-TBD-v0.19.0-epic-a.md`
  - e.g. `docs/pr/PR-TBD-v0.19.0-epic-b-sot-ux.md`

Optional (after PR number is assigned):

- rename to `docs/pr/PR-<pr>-<slug>.md`
  - e.g. `docs/pr/PR-123-sot-template-helper.md`
- rename to `docs/pr/PR-<pr>-<release>-epic-<epic>[-<slug>].md`
  - e.g. `docs/pr/PR-123-v0.19.0-epic-b-sot-ux.md`

The key is that the file exists *before* you open the PR, so the PR body can link to it reliably.

## Workflow

1. Create a new SOT file:
   ```bash
   cargo run -p veil-cli -- sot new \
     --slug sot-template-helper \
     --title "Add PR SOT template helper"
   ```
2. Fill standard sections:
   - SOT / What / Verification / Evidence / Non-goals / Rollback
3. Copy-paste the SOT filename into your PR description:
   ```md
   ### SOT
   - docs/pr/PR-TBD-...
   ```
4. After the PR number is assigned, rename the file if desired:
   ```bash
   cargo run -p veil-cli -- sot rename --pr 123 --path docs/pr/PR-TBD-sot-template-helper.md
   ```
5. Keep the SOT updated as the PR evolves.

## Useful Commands

Dry-run without writing a file:

```bash
cargo run -p veil-cli -- sot new \
  --slug sample \
  --title "Sample" \
  --dry-run
```

Create a PR-numbered SOT directly:

```bash
cargo run -p veil-cli -- sot new \
  --pr 123 \
  --slug sample \
  --title "Sample"
```

Create a release/epic style SOT:

```bash
cargo run -p veil-cli -- sot new \
  --release v0.19.0 \
  --epic A \
  --slug audit-log \
  --title "Audit log updates"
```

`sot new` does not insert the current date by default. Use `--date YYYY-MM-DD` when the date should be part of the record.

## Why SOT?

- Consistent naming + metadata via a deterministic helper.
- Persistent record that survives squash merges.
- Single place for complex verification logs and evidence.
