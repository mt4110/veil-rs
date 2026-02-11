# PR42 SOT (Epic D / Registry v1)
- Scope: Exception Registry v1 schema/format validation + wiring + tests
- Non-goals: expiry enforcement (PR43+)
- Verification:
  - go test ./...
  - nix run .#prverify
  - nix build .#cockpit
  - nix build .#ai-pack
