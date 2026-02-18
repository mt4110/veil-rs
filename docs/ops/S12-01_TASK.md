# S12-01 TASK — A: Verify Chain v1 (implementation)

## Kickoff (docs + status)
- [ ] STATUS: S12-00 -> 100% (Merged), S12-01 -> 1% (WIP)
- [ ] Add PLAN/TASK + PR SOT doc

## Inventory (small steps; no heavy runs)
- [ ] List verify-related entrypoints (cmd/*, internal pkgs)
- [ ] Grep for keywords: verify/signature/allowlist/fingerprint/policy/rotation
- [ ] Write “Invariants v1” list (must-pass / must-fail)

## Implementation (single phase, single PR)
- [ ] Implement minimal verify-chain hardening v1
- [ ] Add tests for must-pass / must-fail + regression lock
- [ ] Ensure deterministic behavior (ordering, time, path)

## Gates
- [ ] go test ./... (PASS)
- [ ] nix run .#prverify (PASS, if available)
