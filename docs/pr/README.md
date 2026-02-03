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

## Workflow (v0.19.0+)

1. Create a new SOT file:
   ```bash
   veil sot new --epic <A|B|...> --slug <short-name>
   # Add --release vX.Y.Z if release inference fails.
   ```
2. Copy-paste the block printed by the command into your PR description:
   ```md
   ### SOT
   - docs/pr/PR-TBD-<release>-epic-<epic>[-<slug>].md
   ```
3. Keep the SOT updated as the PR evolves.
4. (Optional) rename once the PR number exists:
   ```bash
   veil sot rename --pr <number>
   ```

## Why SOT?

- Consistent naming + metadata via `veil sot new` (no manual `cat > ...`).
- Persistent record that survives squash merges.
- Single place for complex verification logs and evidence.

> [!CAUTION]
> **Use the correct `veil` binary**
> When running `veil sot new`, ensure you are using the version of `veil` consistent with the repository tools.
>
> **Recommended:**
> ```bash
> cargo run -p veil-cli -- sot new --epic <EPIC> --slug <SLUG>
> ```
