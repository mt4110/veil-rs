# Japanese Rule Templates

This directory stores inactive Japanese rule templates.

Templates here are not part of the default RulePack. Promote selected files into an executable RulePack only after reviewing false positives, score policy, and manifest entries.

Current template packs:

- `jp_security_templates_1000`: 1000 generated JP security / PII / secret detection templates organized by category and variant.

Promoted opt-in packs:

- `../packs/jp_security_critical`: 37 critical `secret` / `finance` key/value rules promoted from `jp_security_templates_1000`.
