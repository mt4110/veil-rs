---
release: {{release}}
epic: {{epic}}
pr: TBD
status: draft
created_at: {{created_at}}
branch: {{branch}}
commit: {{commit}}
title: {{title}}
---

## Overview
(1-3 lines)

## Goals
- [ ] ...

## Non-Goals
- [ ] ...

## Design
### CLI
- Command: `veil sot new ...`
- Output path: `docs/pr/...`

### Files / Formats
- ...

## Verification
- [ ] `cargo test -p veil-cli`
- [ ] Manual: `veil sot new --dry-run ...`

## Risks / Rollback
- Risks:
- Rollback:

## Audit Notes
- Evidence:
  - Generated SOT file under docs/pr
  - CI logs referencing SOT path
