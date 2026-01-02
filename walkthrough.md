# walkthrough.md — RulePack as Source of Truth (Canonical)

This document is the canonical guide for Veil’s rules architecture and the “Log Scrubbing” workflow.

## TL;DR

- **Rules are canonically defined as RulePacks** (directory + `00_manifest.toml` + `[[rules]]` TOMLs).
- `veil.toml` is primarily an **overlay** (enable/disable, overrides, repo-specific rules).
- For log pipelines, use the **Log RulePack** and fixed placeholders:
  - `<REDACTED:OBSERVABILITY>` / `<REDACTED:SECRET>` / `<REDACTED:PII>`
- In Logs profile, the goal is **safe output**, not strict blocking:
  - **Logs should not fail the pipeline** (masking first, discipline over punishment).

---

## Concepts

### RulePack (canonical)
A RulePack is a directory containing:

- `00_manifest.toml` — pack metadata + deterministic file load order
- one or more TOML files containing `[[rules]]`

A pack is the **source of truth** for rule definitions.

### Config (overlay)
`veil.toml` (and org/user layers) is used for:

- `core.rules_dir` to point to a pack
- masking preferences (mode / placeholder default)
- project-local rules or overrides

---

## Built-in packs (embedded in the binary)

Veil ships with embedded packs:

- `crates/veil/rules/default/` — default scan rules
- `crates/veil/rules/log/` — log scrubbing pack (OBS/SECRET/PII)

These are included at build time.

---

## Using RulePacks in a repo (rules_dir)

Point `core.rules_dir` to a pack directory:

```toml
[core]
rules_dir = "rules/log"
```

Veil resolves `rules_dir` relative to the config file location (repo layer),
then passes an absolute path to core for loading.

---

## Batteries-included: Log Scrubbing workflow

### 1) Generate a repo-local Log RulePack

Run:

```bash
veil init --profile Logs
```

This creates:

- `rules/log/` (RulePack directory)
- `veil.toml` wired with `core.rules_dir = "rules/log"`
- placeholders fixed to:
  - `<REDACTED:OBSERVABILITY>`
  - `<REDACTED:SECRET>`
  - `<REDACTED:PII>`

### 2) Mask STDIN logs safely

Examples:

```bash
# Basic
my-app | veil filter

# With explicit config (global flag)
my-app | veil --config veil.toml filter
```

### 3) Why “OBSERVABILITY” masking exists

Log leaks are not only about secret values.
They often leak the **map** (toolchain, endpoints, vendor names, env keys, paths).
If an attacker gets logs, we don’t want to hand them the wiring diagram.

So the Log Pack masks:

- service/product names (Sentry / Kibana / Fluentd / etc.)
- vendor domains / DSN hostnames
- env key names (SENTRY_DSN, DD_API_KEY, OTEL_*, …)
- well-known config/log paths

### Placeholder policy (fixed to 3 kinds)

Log output uses exactly:

- `<REDACTED:OBSERVABILITY>`
- `<REDACTED:SECRET>`
- `<REDACTED:PII>`

Reason:

- diff is stable
- audits are clean
- humans can read logs without “noise confetti”

Rule detail should be expressed via **tags**, not by inventing new placeholders.

### Overlap policy (winner takes all)

When multiple rules match overlapping spans, Veil applies:

- **winner-takes-all** over the unioned span
- priority is determined by category/operational precedence

This avoids outputs like:

```
<REDACTED><REDACTED><REDACTED>
```

…which destroys log readability.

---

## Customization: how to evolve the pack

Treat `rules/log/` as code:

- add rules in small files (keep regex size manageable)
- keep high-confidence rules separate from aggressive ones
- prefer exact vendor names / env prefixes over generic words

If you need “aggressive masking”:

1. Uncomment `"observability_services_ext.toml"` in `00_manifest.toml` (it is included but disabled by default).
2. Or add your own "ext" file with lower scores and an `aggressive` tag.

---

## Future: signed / remote packs (design intent)

RulePacks are designed to evolve toward:

- remote fetch
- integrity verification (signatures)
- org policy distribution

That’s why RulePack is the canonical unit, not config blobs.

---

# End

If you only remember one thing:
**strong rules are nice, but strong habits win.**
Use Log Pack everywhere logs can escape your boundary.
