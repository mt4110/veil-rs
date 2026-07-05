# Japanese Opt-in RulePacks

This directory stores executable Japanese RulePacks that are not part of the default RulePack.

Use these packs by setting `[core].rules_dir` to the pack directory in a project config. Keep generated template inventory under `rules_ja/templates`; only reviewed, fixture-backed selections belong here.

Current packs:

- `jp_security_critical`: critical `secret` and `finance` key/value rules promoted from `jp_security_templates_1000`.
