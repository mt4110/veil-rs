# S12-01 PLAN — A: Verify Chain v1 (implementation)

## Goal (1-line)
Kill “green-on-broken” in verification paths by turning invariants into tests + gates.

## Scope (v1)
- Define verification invariants (spec = law)
- Implement the smallest enforcement unit
- Lock behavior with regression tests
- Prove via deterministic gates (CI-readable)

## Invariants (must be TRUE)
- If any required artifact is missing or malformed => verification MUST fail (not warn)
- If any checksum/signature/key policy is violated => MUST fail
- Verification output is deterministic (order/time/path independent as much as possible)
- UX: errors are explainable and stable enough for tests

## Plan (pseudo-code)
try:
  inventory current verify routes
  for each route:
    define must-fail cases
    define must-pass cases
  implement minimal checker that enforces invariants
  add tests:
    - pass fixtures
    - tamper fixtures (checksum/signature/key)
  add gates:
    - go test ./...
    - prverify (if available)
catch:
  error: stop and document the mismatch between spec and implementation

## Stop Conditions
- Behavior change without tests => STOP
- New policy without a checker/gate => STOP
- Any path that can silently accept invalid input => STOP
