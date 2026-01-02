# veil/rules — Rule Packs (Default, Remote, Local)

This directory contains **Rule Packs** for `veil`.

A Rule Pack is a versioned bundle of rules (patterns, metadata, and optional validators by name)
that can be merged with other packs to build the **Effective Rule Set** used during scanning.

The design goals are:

* **Determinism**: same inputs → same effective rules, with stable merge order.
* **Transparency**: list/explain can show where each rule came from, and why it wins.
* **Safety**: remote rules are **data-only**; no arbitrary code execution.
* **Evolution**: packs can be versioned, signed, and distributed.

---

## Directory layout

```
veil/
  rules/
    README.md
    default/
      00_manifest.toml
      *.toml
```

* `default/` holds the **built-in default rule pack**, stored as data files.
* `00_manifest.toml` defines **load order** and **pack metadata** (including future signature policy).

---

## Concepts

### Rule ID (canonical)

Rules are identified by a canonical ID, recommended format:

* `namespace.category.name` (dot-separated), e.g. `creds.aws.access_key_id`
* IDs must be globally unique after merge.

Alias IDs (legacy snake_case like `aws_access_key_id`) may exist, but must normalize to canonical IDs.
Normalization is performed by the engine, not by TOML authors.

### Rule Pack

A Rule Pack is a set of TOML files that collectively define rules.

Each TOML file may define:

* `[pack]` metadata (optional per file, but recommended)
* `[[rules]]` entries (rule definitions)

A pack has:

* `pack_id` (string)
* `pack_version` (integer, monotonic)
* `schema_version` (integer)
* `source` (builtin/default_files/remote/local)
* optional signature policy (see below)

### Effective Rule Set

The engine merges multiple sources in a strict order to produce the Effective Rule Set.

Recommended merge order (highest priority wins on conflicts):

1. **builtin** (legacy Rust-embedded defaults, transitional)
2. **default_files** (`veil/rules/default`, this directory)
3. **remote** (downloaded rule packs)
4. **local** (`veil.toml` rule overrides / additions)

This order can be implemented as “apply in order, later overwrites earlier”.
The engine should record provenance for `veil rules list/explain`.

---

## Deterministic load order

Within a single directory pack (`default/`), load order must be stable and explicit.

`00_manifest.toml` defines:

* Which files are included
* The exact sequence they are applied
* Optional constraints (e.g., required schema version)

If the manifest is missing, the engine **must not** guess order by filesystem traversal.
(At most, it may sort filenames lexicographically, but manifest is the canonical mechanism.)

---

## Versioning policy

There are two version numbers:

### `schema_version`

Defines the TOML schema used to interpret the pack files.
Increment only when the TOML structure/semantics change (breaking).

### `pack_version`

Defines the content evolution within the same schema.
Increment when rules change (add/remove/modify), even if schema stays the same.

Both are integers.

Recommended semantics:

* `schema_version`: change rarely, breaking changes only.
* `pack_version`: change frequently, content iteration.

---

## Remote rules and signatures (future-proof)

Remote Rule Packs must be treated as **untrusted input**.

Principles:

* Packs are **data-only** (TOML or equivalent). No embedded scripts.
* Validators are referenced by **name** (e.g., `"luhn"`) and resolved only to a known allowlist.

Signature policy (planned):

* Remote pack distribution provides:

  * `manifest` (pack metadata)
  * `files` (pack content)
  * `signature` over a canonical digest set (e.g., SHA-256 over each file, plus manifest)
* The engine verifies signatures against a configured trust store.

Trust models (choose per org):

* **Pinned key(s)**: only accept packs signed by listed public keys.
* **Pinned digest(s)**: accept only specific known hashes.
* **TOFU (Trust On First Use)**: accept first signature, pin thereafter (not recommended for high-security).

Manifest will include fields to express intent (e.g., `signature.required = true`),
but the effective enforcement is always decided by local policy (config).

---

## Minimal TOML structure (reference)

A typical rule file:

```toml
[pack]
id = "default.cloud_keys"
version = 1
schema_version = 1
description = "Cloud credential patterns (high-confidence)"

[[rules]]
id = "creds.aws.access_key_id"
description = "AWS Access Key ID"
pattern = '''\b(AKIA|ASIA)[0-9A-Z]{16}\b'''
severity = "HIGH"
score = 85
category = "secret"
tags = ["credential","cloud","aws"]
```

Notes:

* `pattern` is recommended with `'''` for readability.
* Additional fields (context, allow/deny, etc.) may be added under the same `schema_version`
  if backwards-compatible; otherwise bump `schema_version`.

---

## Non-goals (for now)

* No dynamic code execution in packs.
* No auto-updating remote packs without explicit user/org policy.
* No implicit merging rules based on filename conventions; manifest is the source of truth.

---

## Operational guidance

* Prefer small, themed files (cloud keys / PII JP / formats / entropy).
* Keep `00_manifest.toml` stable and reviewable.
* Treat pack updates as security-sensitive changes:

  * review diffs
  * run golden tests comparing effective rulesets
  * record provenance in `veil rules list/explain`
