# Local Storage Policy v1

**Canonical reference for `.local/` directory management in veil-rs.**

---

## Purpose

`.local/` is the **scratch space** for local development artifacts (prverify reports, review bundles, observations, caches). It is gitignored and never committed.

Without a policy, `.local/` accumulates indefinitely and causes:
- Disk bloat
- Stale evidence confusing reproducibility checks
- Inability to identify "newest valid" artifacts

---

## Directory Layout

| Path                                              | Category                            | Keep Policy                           |
| ------------------------------------------------- | ----------------------------------- | ------------------------------------- |
| `.local/prverify/`                                | Prverify reports (`prverify_*.md`)  | Keep last **5** bound to any HEAD SHA |
| `.local/review-bundles/`                          | Review bundle archives (`*.tar.gz`) | Keep last **3** per mode (strict/wip) |
| `.local/obs/`                                     | Observation logs (`s12-*_*.txt`)    | Keep last **10** per phase prefix     |
| `.local/cache/`                                   | Build caches                        | Managed by build tools (do not GC)    |
| `.local/bin/`                                     | Local binaries                      | Managed manually (do not GC)          |
| `.local/archive/`                                 | Explicitly archived items           | Never delete automatically            |
| `.local/evidence/`                                | Evidence snapshots                  | Keep last **5**                       |
| `.local/handoff/`                                 | Handoff docs                        | Keep last **3**                       |
| `.local/lock_backup/`                             | Lock file backups                   | Keep last **5**                       |
| `.local/obs/`                                     | All observation dirs/files          | Keep last **10** (by mtime)           |
| Other files at `.local/*.{txt,tsv,stderr,stdout}` | Misc scratch                        | GC candidates (older than 7 days)     |

---

## Retention Rules

### Default: Never Delete Without Explicit Flag

```
--mode dry-run (DEFAULT): list candidates, delete nothing
--mode plan:              list with sizes and reasons
--mode apply + --apply:   both flags required (double-lock)
```

**Safety First**: any operation that removes files requires both:
- `--mode apply` (explicit intent)
- `--apply` flag (double confirmation)

Any tool invocation with only one of these flags must print `SKIP: apply requires both --mode apply AND --apply` and exit cleanly with `stop=0`.

### Age-Based Candidates

Files/dirs older than **7 days** (mtime) are candidates for removal when they exceed retention count.

### Size Limit Warning

If any category exceeds **500 MB** total, print `WARN: dir=<dir> size=<N>MB exceeds 500MB`.

---

## What is NEVER Deleted Automatically

- `.local/archive/` — explicitly archived; manual only
- `.local/bin/` — local binaries
- `.local/cache/` — managed by build tools
- Any file/dir with `KEEP` in the name

---

## CI Policy

- `--mode dry-run` is safe in CI (printing only)
- `--mode apply` with `--apply` is **prohibited** in CI pipelines
- GC is a **developer tool**, not a CI gate

---

## Output Contract (Stopless)

All output to stdout follows:

```
OK: dir=<dir> count=<N> newest=<ts>        # per-dir summary
WARN: dir=<dir> size=<N>MB exceeds 500MB   # optional warnings
SKIP: apply requires both --mode apply AND --apply  # if only one lock
ERROR: <detail>                            # on unexpected failure
OK: phase=end stop=<0|1>                   # always last
```

Process always exits 0. Consumers parse `stop=` from stdout.
