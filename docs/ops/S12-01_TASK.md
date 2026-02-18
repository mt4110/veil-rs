# S12-01 TASK — A: Verify Chain v1 (implementation)

## Kickoff (docs + status)
- [x] STATUS: S12-00 -> 100% (Merged), S12-01 -> 1% (WIP)
- [x] Add PLAN/TASK + PR SOT doc

## Inventory (small steps; no heavy runs)
- [x] List verify-related entrypoints (cmd/*, internal pkgs)
- [x] Grep for keywords: verify/signature/allowlist/fingerprint/policy/rotation
- [x] Write “Invariants v1” list (must-pass / must-fail)

## Implementation (Unify Chain)
- [ ] **Fix `prverify` output**: Change `git rev-parse --short=12 HEAD` to `git rev-parse HEAD` (40-char).
- [ ] **Update Evidence**: Inject 40-char SHA into local `prverify` evidence to pass strict check.
- [ ] **Verify Fix**: Run `reviewbundle create --mode strict` (MUST PASS).

## Implementation (Tests & Hardening)
- [ ] Implement defined Invariants v1 in `cmd/reviewbundle/verify_*_test.go`.
- [ ] Add regression tests:
    - [ ] `strict` + no evidence -> FAIL
    - [ ] `strict` + evidence w/o SHA40 -> FAIL
    - [ ] `strict` + evidence w/ SHA40 -> PASS
- [ ] Ensure deterministic behavior.

## Gates
- [ ] go test ./... (PASS)
- [ ] nix run .#prverify (PASS, if available)
