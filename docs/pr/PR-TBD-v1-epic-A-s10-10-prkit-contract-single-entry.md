# Fix(prkit): Contract Single-Entry & Evidence Convergence (S10-10)

## Overview
This PR consolidates the entry points for PRKit/Rituals to a single contract and stabilizes evidence generation by neutralizing nondeterministic factors.

## SOT (Source of Truth)
- Plan: docs/ops/S10_10_PLAN.md
- Task: docs/ops/S10_10_TASK.md
- Evidence: PENDING (to be filled after prverify)

## Changes
- DELEGATE: cmd/prkit/main.go delegates to internal/prkit/cli.go (new entry point)
- CONSOLIDATE: Orchestration flows through internal/prkit/run.go
- NORMALIZE: time.Now/USER normalized for stable evidence

## Verification
- go test ./... -count=1
- nix run .#prverify
