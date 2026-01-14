# Release Info
Version: <!-- e.g., v0.18.0 -->

# SOT
<!-- REQUIRED -->
SOT: docs/pr/PR-TBD-<short>.md

<!--
Run this command in your terminal to create the SOT file:

cat > docs/pr/PR-TBD-<short>.md <<'EOF'
# PR-TBD-<short>: [Title]

## Why (Background)
- Why now? What pain does this solve?

## Summary
- 2–3 lines.

## Changes
- [ ]

## Non-goals (Not changed)
- [ ]

## Impact / Scope
- CLI: [ ]
- CI: [ ]
- Docs: [ ]
- Rules: [ ]
- Tests: [ ]

## Verification

### Commands
```bash
# example
cargo test --workspace
```

### Notes / Evidence
- [ ]

## Rollback
- How to revert safely (1–2 lines).
EOF
-->

# Release Checklist
- [ ] CHANGELOG.md updated
- [ ] Version bumped in Cargo.toml
- [ ] Tag signed (`git tag -s v0.18.0 -m "v0.18.0"`)
- [ ] Verification Command: `git verify-tag v0.18.0`

# Verification
- [ ] CI/CD Green
