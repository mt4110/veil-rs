# COCKPIT_SWITCH_SNIPPETS

This file is copy‑paste ready:
- Wrapper scripts (deprecated, wrapper-only)
- CI workflow snippet (`ai_guardrails.yml`)
- “Ambi ↔ Sora sync prompt” (anti-derailment)

---

## 1) Wrapper scripts (exact text)

### scripts/ai/gen.sh
```bash
#!/usr/bin/env bash
# DEPRECATED: Wrapper only. Delegates to Cockpit (Nix + Go).
exec nix run .#gen -- "$@"
```

### scripts/ai/check.sh
```bash
#!/usr/bin/env bash
# DEPRECATED: Wrapper only. Delegates to Cockpit (Nix + Go).
exec nix run .#check -- "$@"
```

### scripts/ai/status.sh
```bash
#!/usr/bin/env bash
# DEPRECATED: Wrapper only. Delegates to Cockpit (Nix + Go).
exec nix run .#status -- "$@"
```

---

## 2) CI workflow snippet (ai_guardrails.yml)

> Notes:
> - `fetch-depth: 0` to ensure git context is available.
> - Determinate Systems installer + magic cache for Nix parity + speed.
> - Runs `nix run .#check` as the single source of truth.

```yaml
name: ai-guardrails

on:
  pull_request:
  push:
    branches: [main]

permissions:
  contents: read

jobs:
  ai_guardrails:
    runs-on: ubuntu-latest
    steps:
      - name: Checkout
        uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Install Nix
        uses: DeterminateSystems/nix-installer-action@v15

      - name: Nix Cache
        uses: DeterminateSystems/magic-nix-cache-action@v7

      - name: Run Cockpit Guardrails (Parity)
        run: |
          nix run .#check

      # Optional: metrics summary (safe heredoc; no YAML indentation foot-guns)
      - name: Metrics Summary (best effort)
        if: always()
        shell: bash
        run: |
          cat <<'EOF'
          Metrics policy:
          - Emitted by veil-aiux on failures to dist/metrics/...
          - CI should not upload AI_PACK (*.txt). Only markdown artifacts if needed.
          EOF
```

---

## 3) “Ambi ↔ Sora Sync Prompt” (anti-derailment)

Paste this to Ambi when you want strict alignment.

```text
You are Ambi (implementation). You must follow docs/ai/COCKPIT_SPEC.md (v1.2) as law.

NON-NEGOTIABLES:
- Single source of truth: Go engine veil-aiux.
- No auto-repair: never modify templates/docs/code automatically.
- Contract: dist/publish/<VER>/ has exactly 4 artifacts, AI_PACK is .txt only.
- Exit codes: 0/2/3/4/5 per spec.
- Wrappers are deprecated and wrapper-only (exec nix run ...). No logic.
- CI must run: nix run .#check (fetch-depth: 0).

ANTI-DERAILMENT:
- If unsure -> STOP and ask for clarification (Red Lamp).
- No “helpful” implicit behavior changes.
- Prefer explicit, copy-paste deliverables: wrapper scripts, YAML snippet, and minimal diffs.

DELIVERABLES FOR THIS STEP:
1) The exact wrapper script texts for scripts/ai/{gen,check,status}.sh
2) The CI workflow YAML snippet (ai_guardrails.yml) using DeterminateSystems installer + magic cache + nix run .#check
3) Any change must be spec-compliant. If not, state “RED LAMP” and refuse.
```

---

## 4) Quick sanity commands

```bash
# Local
./scripts/ai/check.sh
./scripts/ai/gen.sh --version v0.14.0 --clean
./scripts/ai/status.sh --version v0.14.0

# Direct
nix run .#check
nix run .#gen -- --version v0.14.0 --clean
nix run .#status -- --version v0.14.0
```
