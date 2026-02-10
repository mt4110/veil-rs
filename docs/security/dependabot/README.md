# Dependabot Remediation Process

## Operational Rules (RunAlways)

1. **Entrance Check**:
   Before starting work, ensure the repository is clean and verifying:
   ```bash
   nix run .#prverify
   ```

2. **Deterministic Inputs**:
   Use Nix to pin dependencies and tools. Avoid relying on global toolchains unless necessary.

3. **One-Scroll Recovery**:
   If a command fails, the cause and next action should be visible in the last screen of output.

## Remediation Strategy (Minimum PR Principle)

- **1 PR = 1 Vulnerability** (or a small cluster of related low-risk updates).
- **Order of Operations**:
  1. Low Risk / Patch Updates (Fix available, `cargo update -p <package>`)
  2. Medium Risk / Minor Updates (Check compatibility)
  3. High Risk / Breaking Changes (Isolate in dedicated PRs)
- **Evidence Required**:
  - `cargo tree` (before/after)
  - `cargo test --workspace` (must pass)
  - `nix run .#prverify` (must pass)

## Handling Exceptions

Per the invariant "Don't ignore, triage":

- If a vulnerability cannot be fixed (no patch, breaking change too large for now):
  - Add an entry to the current Triage Report `docs/security/dependabot/triage_YYYYMMDD.md`.
  - Open a GitHub Issue detailing the blocker and setting an expiry/review date.
  - Do not leave it "open" without a plan.
