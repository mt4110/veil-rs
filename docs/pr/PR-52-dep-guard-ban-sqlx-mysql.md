# PR42 SOT (Epic D / Registry v1)
- Scope: Exception Registry v1 schema/format validation + wiring + tests
- Non-goals: expiry enforcement (PR43+)
- Verification:
  - go test ./...
  - nix run .#prverify
  - nix build .#cockpit
  - nix build .#ai-pack

Latest prverify report: docs/evidence/prverify/prverify_20260211T021046Z_fe0205b.md
