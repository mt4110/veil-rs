# JP Security Critical RulePack

Opt-in RulePack for critical Japanese security and finance key/value findings.

## Scope

- Source: `crates/veil/rules_ja/templates/jp_security_templates_1000`
- Selection: `category in {secret, finance}`, `variant = kv`, `severity = critical`, `score >= 90`
- Rule count: 37
- Categories: `secret` 28, `finance` 9

This pack does not expand the default RulePack. Projects must opt in with `[core].rules_dir`.
For normal project use, copy or vendor this pack into the consuming repository and point
`rules_dir` at that project-local directory. A path that only exists inside the `veil-rs`
checkout is skipped when the CLI runs from another project.

```toml
[core]
rules_dir = "rules/jp_security_critical"
```

## Promotion Command

```bash
cargo run -q -p veil-cli -- rules promote-templates \
  --templates-dir crates/veil/rules_ja/templates/jp_security_templates_1000 \
  --out-dir crates/veil/rules_ja/packs/jp_security_critical \
  --force \
  --category secret \
  --category finance \
  --variant kv \
  --severity critical \
  --min-score 90 \
  --pack-id veil.jp.security.critical
```

## Non-goals

- No `lv`, `schema`, or `leak` variants.
- No address or name validator implementation.
- No default RulePack manifest changes.

The `lv` templates intentionally remain inactive for now because `kv` and `lv` overlap on ordinary `key: value` lines and would produce duplicate findings.
