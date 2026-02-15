# [PR-74] Fix(prkit): Contract Single-Entry & Evidence Convergence (S10-10)

## Overview
This PR consolidates the entry points for PRKit/Rituals to a single contract and stabilizes evidence generation by neutralizing nondeterministic factors.

## SOT (Source of Truth)
- Plan: docs/ops/S10_10_PLAN.md
- Task: docs/ops/S10_10_TASK.md
- Evidence: [docs/evidence/prverify/prverify_20260215T030002Z_013a20b.md](docs/evidence/prverify/prverify_20260215T030002Z_013a20b.md)

## Changes
- DELEGATE: cmd/prkit/main.go delegates to internal/prkit/Run.
- CONSOLIDATE: Orchestration flows through internal/prkit/cli.go (migrated from main.go).
- HARDEN: ScaffoldSOT now accepts io.Writer to follow the unified contract.
- NORMALIZE: time.Now (via prkit.Now) and USER env normalized for stable evidence.

## Verification
- go test ./... -count=1 (Passed)
- nix run .#prverify (Passed)
- Determinism proved via TestDeterminism in internal/prkit (captured in evidence).
