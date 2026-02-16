# Release Info
Version: <!-- e.g., v0.18.0 -->

# SOT
<!-- REQUIRED: Source of Truth document. Create it using the command below, then paste the output block here. -->

### SOT
- docs/pr/PR-TBD-<release>-epic-<epic>[-<slug>].md

<!--
Manual SOT creation:
1) Create a SOT file under `docs/pr/`:
   - `docs/pr/PR-<PR_NUMBER>-<slug>.md`
2) Fill standard sections:
   - SOT / What / Verification / Evidence
3) Commit + push, then rerun the gate (CI / prverify).
-->

# Release Checklist
- [ ] CHANGELOG.md updated
- [ ] Version bumped in Cargo.toml
- [ ] Tag signed (`git tag -s v0.18.0 -m "v0.18.0"`)
- [ ] Verification Command: `git verify-tag v0.18.0`

# Verification
- [ ] CI/CD Green
