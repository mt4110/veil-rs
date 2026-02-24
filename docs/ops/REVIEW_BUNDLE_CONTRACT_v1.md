# Review Bundle Contract v1.1

**Canonical reference for all review bundle producers and consumers.**

---

## Overview

A review bundle is a deterministic `.tar.gz` archive created by `reviewbundle create`.
This document defines what is **guaranteed** (must pass `reviewbundle verify`) and what is **not guaranteed**.

## contract_version

- Single source of truth: `review/meta/contract.json` field `"contract_version": "1.1"`
- Consumers MUST read `contract_version` from `contract.json` only
- No other file duplicates this field

## Required Files (Layout)

Every bundle MUST contain exactly these paths:

| Path                            | Description                                                                |
| ------------------------------- | -------------------------------------------------------------------------- |
| `review/INDEX.md`               | Human-readable index                                                       |
| `review/meta/contract.json`     | Machine-readable contract (contract_version, mode, head_sha, epoch_sec, …) |
| `review/meta/SHA256SUMS`        | Deterministic manifest of all files                                        |
| `review/meta/SHA256SUMS.sha256` | Seal (SHA256 of SHA256SUMS)                                                |
| `review/patch/series.patch`     | Git patch from base_ref to HEAD                                            |

Optional (required only in `strict` mode):

| Path                                     | Description                         |
| ---------------------------------------- | ----------------------------------- |
| `review/evidence/prverify/prverify_*.md` | Prverify evidence bound to HEAD SHA |
| `review/meta/warnings.txt`               | Present if `warnings_count > 0`     |

## Required contract.json Fields

```json
{
  "contract_version": "1.1",
  "mode":             "<strict|wip>",
  "repo":             "veil-rs",
  "epoch_sec":        <unix timestamp — deterministic, from SOURCE_DATE_EPOCH or git show>,
  "base_ref":         "main",
  "head_sha":         "<40 hex chars>",
  "warnings_count":   <int>,
  "evidence":         { "required": <bool>, "present": <bool>, "bound_to_head": <bool>, "path_prefix": "review/evidence/" },
  "tool":             { "name": "reviewbundle", "version": "1.0.0" }
}
```

## Normalization Rules

### Tar archive
- All entries sorted **lexicographically ascending** by path
- All mod times equal to `epoch_sec` (zero nanoseconds)
- All uid/gid = 0, uname/gname = ""
- Regular files: mode 0644 or 0755; directories: 0755
- No PAX time keys (mtime/atime/ctime)
- No xattrs (LIBARCHIVE.*,  SCHILY.xattr.*)
- Gzip header: OS=255, Name="", Comment="", Extra=[]

### SHA256SUMS
- One line per file: `<sha256hex>  <path>`
- Sorted by path (same order as tar)
- Sealed by `SHA256SUMS.sha256`

### Timestamps
- `epoch_sec` comes from `SOURCE_DATE_EPOCH` env var if set; otherwise `git show -s --format=%ct HEAD`
- This ensures reproducible builds

## Compatibility Policy

- **Forward**: New producers MUST emit `contract_version: "1.1"` and all required fields
- **Backward**: Consumers (verify) accept bundles created by any producer that emits `contract_version: "1.1"`
- Breaking changes require a version bump to `"1.2"` (or major: `"2.0"`)
- Current verifier (`VerifyBundle`) enforces exactly `"1.1"` — any other value is rejected

## What is NOT Guaranteed

- Contents of `review/INDEX.md` (human-readable, no schema)
- Order or number of evidence files (only that ≥1 binds to HEAD in strict mode)
- Size of individual files (only that meta/evidence files > 4MB are skipped in SHA256SUMS verification)
- Future fields in `contract.json` (additive fields allowed without version bump)

## Verification Exit Contract (Stopless)

`reviewbundle verify <path>` output:

```
# on success:
OK: contract=1.1 mode=<mode> head=<sha> epoch=<ts>
PASS: bundle verified
OK: phase=end stop=0

# on failure:
ERROR: <category> path=<path> detail=<msg>
OK: phase=end stop=1
```

Process always exits 0. Consumers MUST parse `stop=` value, NOT exit code.
