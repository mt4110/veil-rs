# RulePack Update Flow

This runbook defines the v1 RulePack update flow for Japanese enterprise deployments. It avoids
automatic network updates and treats RulePack changes as security-sensitive releases.

## Purpose

RulePack updates must be deterministic, reviewable, and reversible. The default flow is:

```text
candidate pack -> offline verification -> pinned digest update -> atomic promote -> scan/doctor
```

This is an operational flow, not an automatic downloader.

## Non-Goals

- Do not fetch RulePacks automatically from the network.
- Do not enable `remote_rules_url` by default.
- Do not trust a RulePack only because it came from an internal URL.
- Do not mutate an active `rules_dir` in place.
- Do not implement `pinned_keys` or TOFU in v1.

## Directory Layout

```text
rules/
  active/              # current rules_dir target
  candidates/
    2026-07-11-pack/   # newly reviewed candidate
  archive/
    2026-07-01-pack/   # rollback copy
```

`[core] rules_dir = "rules/active"` points only to the promoted active pack.

## Candidate Intake

1. Receive or generate a candidate RulePack outside `rules/active`.
2. Confirm the candidate contains `00_manifest.toml`.
3. Confirm `[signature]` uses `trust_model = "pinned_digests"` and `digest_algorithm = "sha256"`.
4. Review the RulePack diff as a security-sensitive change.
5. Run the loader against the candidate pack before promotion.

The candidate may come from a vendor, a private repository, or `veil rules promote-templates`.
The source does not change the verification requirement.

## Verification Gate

Before promotion, verify:

- `load_rule_pack(candidate)` succeeds.
- The candidate manifest has `signature.enabled = true` or `signature.required = true`.
- The computed RulePack digest is present in `signature.pinned_digests`.
- Unsupported trust models fail closed.
- The candidate does not rely on network access.
- Golden fixture scans or representative repository scans are reviewed.

The current v1 implementation enforces the pinned digest check when the RulePack is loaded. This
means CLI, LSP, and Local Audit UI share the same gate when they use the same `rules_dir`.

## Promotion

Promotion is a filesystem operation:

1. Copy the current `rules/active` directory to `rules/archive/<timestamp-or-version>`.
2. Copy or move the verified candidate to a temporary path next to `rules/active`.
3. Rename the temporary path into `rules/active` atomically where the filesystem supports it.
4. Run `veil rules list` or `veil scan --preset ...` with the same org/repo config used in CI.
5. Record the promoted digest, pack id, pack version, reviewer, and approval ticket.

Do not edit files directly inside `rules/active`.

## Rollback

Rollback is the reverse promotion:

1. Stop using the failed active pack.
2. Promote the previous archived pack back to `rules/active`.
3. Re-run the same verification command.
4. Record the rollback reason and the restored digest.

Rollback must not delete the failed pack until it has been reviewed.

## Audit Record

Retain:

- candidate source reference
- reviewer and approval ticket
- pack id / version / schema version
- promoted pinned digest
- verification command and result
- scan fixture or representative scan result
- rollback target, if any

## Future Automation Boundary

A future `veil rules update` command may automate parts of this flow, but it must preserve these
properties:

- explicit user or organization opt-in
- no default network access
- staging before promotion
- pinned digest or stronger signature verification before activation
- atomic promotion and rollback
- no upload of source, PII, findings, or Evidence contents
