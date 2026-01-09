# Dogfooding Check List (v0.17.0)
Version: `v0.17.0-rc.2`

- [⚪︎ ] Cmd: `veil --version`
  - Expect: `0.17.0-rc.2`
  - Actual: ____________________
- [⚪︎] Cmd: `veil init --wizard`
  - Expect: Interactive success
  - Actual: ____________________
- [⚪︎] Cmd: `veil init --profile Logs`
  - Expect: "Log RulePack" output appears
  - Actual: ____________________
- [⚪︎] Cmd: `touch veil.toml && veil init`
  - Expect: "Veil configuration already exists" (No Logs output if existing)
  - Actual: ____________________
- [⚪︎] Cmd: `veil init --ci github`
  - Expect: `.github/workflows/veil.yml` contains `cargo install ... --tag v0.17.0`
  - Actual: ____________________