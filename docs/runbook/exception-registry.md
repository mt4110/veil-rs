# Runbook: Exception Registry

## Overview
This runbook covers the operation of the **Exception Registry** stored in:

- `ops/exceptions.toml`

The registry is validated by the repository’s validator rules (required fields, invariants, expiry checks).

---

## Command forms

Recommended (deterministic):

```bash
nix run .#veil -- exceptions <subcommand> [args...]
```

If you're already inside `nix develop` (and `veil` is on PATH), you may use:

```bash
veil exceptions <subcommand> [args...]
```

---

## Common tasks

### List entries

```bash
nix run .#veil -- exceptions list
```

Filter by status:

```bash
nix run .#veil -- exceptions list --status active
nix run .#veil -- exceptions list --status expiring_soon
nix run .#veil -- exceptions list --status expired
```

### Show one entry

```bash
nix run .#veil -- exceptions show <id>
```

---

## Failure UX (prverify)

When `nix run .#prverify` fails due to exception registry issues, it should provide:

- **Reason**: what rule failed
- **Fix**: what to edit
- **Next**: the exact command to run next

Example flow:

1) Identify:
```bash
nix run .#veil -- exceptions list --status expired
```

2) Fix:
- Edit `ops/exceptions.toml` to correct fields or renew expiry.
- Rules are enforced by the validator (required fields, invariants, expiry checks).

3) Validate:
```bash
nix run .#prverify
```

Repeat until PASS.

---

## Operator loop (recommended)

- Regularly check upcoming expirations:
```bash
nix run .#veil -- exceptions list --status expiring_soon
```

- Keep expiries short and justified.
- Avoid “forever exceptions”. If something needs to be permanent, it should become a policy rule or a baseline.

---

## Notes

- This runbook is the source of truth for how to operate the registry.
- Keep command examples deterministic unless explicitly inside `nix develop`.
