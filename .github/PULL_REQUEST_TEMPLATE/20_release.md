# Release Info
Version: <!-- e.g., v0.18.0 -->

# SOT
<!-- REQUIRED -->
SOT: docs/pr/PR-TBD-<short>.md

<!--
Run this command in your terminal to create the SOT file:

# v0.19.0+
veil sot new --epic A --slug <short>

# Then copy-paste the output "### SOT - docs/pr/..." below.
-->

# Release Checklist
- [ ] CHANGELOG.md updated
- [ ] Version bumped in Cargo.toml
- [ ] Tag signed (`git tag -s v0.18.0 -m "v0.18.0"`)
- [ ] Verification Command: `git verify-tag v0.18.0`

# Verification
- [ ] CI/CD Green
