# PR SOT: Strict Ritual Capsule v1

## SOT
- Scope: S12-03 Strict Ritual Capsule (reviewbundle create strict)
- PR: #84
- Branch: s12-03-strict-capsule-v1
- Deliverables:
  - cmd/reviewbundle/create.go
  - cmd/reviewbundle/capsule_test.go
  - docs/ops/STATUS.md (S12-02/S12-03 pointers)
  - docs/evidence/ops/obs_20260219_s12-03.md

## What
- Add strict capsule path in reviewbundle create:
  - auto evidence resolution (prverify report bound to HEAD)
  - optional heavy prverify
  - optional autocommit
- Add capsule-focused Go test coverage
- Update ops docs/evidence pointers for S12-02/S12-03

## Verification
- go test ./... (PASS)
- nix run .#prverify (PASS or SKIP with reason)
- CI required checks green

## Evidence
- obs: docs/evidence/ops/obs_20260219_s12-03.md
- prverify: <FILL path + sha>
- review bundle: <FILL tar + sha>
