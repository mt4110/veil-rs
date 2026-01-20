# Release Info
Version: <!-- e.g., v0.18.0 -->

# SOT
<!-- REQUIRED: Source of Truth document. Create it using the command below, then paste the output block here. -->

### SOT
- docs/pr/PR-TBD-<release>-epic-<epic>[-<slug>].md

<!--
Run this command in your terminal to create the SOT file:

# v0.19.0+
veil sot new --epic <A|B|...> --slug <short>

# Then copy-paste the output block here (it prints the exact "### SOT" section).
-->

# Release Checklist
- [ ] CHANGELOG.md updated
- [ ] Version bumped in Cargo.toml
- [ ] Tag signed (`git tag -s v0.18.0 -m "v0.18.0"`)
- [ ] Verification Command: `git verify-tag v0.18.0`

# Verification
- [ ] CI/CD Green
