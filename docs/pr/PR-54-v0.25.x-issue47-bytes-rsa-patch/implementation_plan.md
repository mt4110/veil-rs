# PR54 Implementation Plan — Issue #47 (bytes/rsa patch updates)

## 🎯 Objective

Apply targeted patch updates (bytes + rsa) to reduce Dependabot alert surface with minimal risk, while preserving determinism and auditability.

Targets:
- bytes: 1.11.0 → 1.11.1
- rsa:   0.9.9  → 0.9.10

## 🧱 Constraints / Invariants (must hold)

- Diff is minimal (prefer lockfile-only).
- Do not bump other crates unless resolver forces it; if forced, document it explicitly.
- Always Run Contract is satisfied:
  - SOT + plan/task present
  - prverify PASS
  - evidence archived
  - docs doc-links guard not violated

## 🧩 Strategy

### Core approach
1) Update exactly the two crates using `--precise` (avoid “latest drift” surprises).
2) Validate:
   - dependency graph sanity (optional but helpful)
   - tests
   - prverify
3) Archive prverify output to docs/evidence
4) Point SOT to the archived evidence

### Why `--precise`
- Determinism: prevents accidental upgrade to a newer patch/minor if available.
- Reviewability: diff matches intended change exactly.

## 🛠️ Step-by-step

### 0. Branch + doc scaffolding
- Create a new PR branch for PR54
- Add SOT / plan / task docs first (so contract exists early)

### 1. Pre-check (optional but recommended)
- Confirm current lock has the old versions
- Confirm whether bytes/rsa are direct or transitive

### 2. Apply updates (lockfile-centered)
- Run:
  - `cargo update -p bytes --precise 1.11.1`
  - `cargo update -p rsa --precise 0.9.10`

### 3. Validate (local)
- `cargo test --workspace`
- `nix run .#prverify`

### 4. Evidence archival (deterministic + docs-safe)
- Identify the latest prverify report under `.local/prverify/`
- Copy it into `docs/evidence/prverify/` using a UTC timestamp + sha7 naming
- If doc-links guard fails due to a file-url string inside the report:
  - sanitize only the offending sequences (keep the report meaning)

### 5. Finalize SOT
- Update:
  - `Latest prverify report:` path in SOT

### 6. Commit / push / PR / merge
- One commit is acceptable if clean; two commits (docs scaffold + lock update) is also fine.
- Ensure PR description references Issue #47.

## 🧯 Failure Handling (IF / THEN)

### IF `cargo update --precise` fails
THEN:
- run `cargo update -p <crate>` without `--precise` to inspect what cargo wants
- check whether the crate is in the graph:
  - `cargo tree -i bytes` / `cargo tree -i rsa`
- record the reason in PR notes before proceeding

### IF compilation/test fails after patch update
THEN:
- treat as unexpected; patch updates *should* be compatible
- make the smallest code fix necessary, limited to the failing callsite
- do not refactor; do not reformat
- rerun tests + prverify and keep evidence

### IF prverify fails
THEN:
- follow the printed gate logs
- fix only what is needed for PASS
- rerun prverify; archive final PASS evidence only (keep intermediate logs local)

## ✅ Done Definition

- lockfile reflects only intended patch bumps (or documented forced resolver moves)
- tests + prverify are green
- evidence archived + SOT updated
- PR ready to merge and closes/advances Issue #47
