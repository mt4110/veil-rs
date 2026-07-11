# Enterprise Policy Templates

These templates are organization-layer `veil.toml` examples for Japanese enterprise deployments.
Use them through `VEIL_ORG_CONFIG` or the standard org config locations documented in
`docs/design/config_layers.md`.

They are intentionally not presets. Built-in presets remain the base layer selected by the user
or workflow, while these policy templates express organization defaults that can still be
overridden by repository config in the current v1 model.

## Usage

```bash
export VEIL_ORG_CONFIG=/path/to/enterprise-fintech.toml
veil scan . --preset fintech-jp
veil lsp --preset fintech-jp
```

Use the same `VEIL_ORG_CONFIG` for CLI, LSP, and Local Audit UI entrypoints so all surfaces see the
same thresholds, masks, limits, and rule score adjustments.

## Templates

| Template | Intended Use |
|---|---|
| `enterprise-standard.toml` | General Japanese corporate code and documentation repositories. |
| `enterprise-fintech.toml` | Finance, payment, accounting, KYC, and settlement systems. |
| `enterprise-gov.toml` | Government, municipality, public-sector, and regulated citizen-data systems. |
| `enterprise-si-vendor.toml` | SIer and outsourced development repositories with mixed customer data. |
| `enterprise-logs.toml` | Log and observability repositories using `logs-jp` and log RulePacks. |

## Contract

- Do not place `preset` in these templates until `CoreConfig.preset` is implemented.
- Do not use `severity` in new rule overrides. Use `base_score`.
- Keep `remote_rules_url` unset for default enterprise templates. Signed remote RulePack delivery is
  tracked separately by the Phase 7 signature and update-flow tasks.
- Treat these files as starting points. Security teams should tune score thresholds after reviewing
  baseline findings for their repositories.
