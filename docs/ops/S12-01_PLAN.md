# S12-01 PLAN — A: Verify Chain v1 (implementation)

## Goal (1-line)
Kill “green-on-broken” in verification paths by turning invariants into tests + gates.

## Scope (v1)
- Define verification invariants (spec = law)
- Implement the smallest enforcement unit
- Lock behavior with regression tests
- Prove via deterministic gates (CI-readable)

## Invariants v1 (Law = Fixed by Tests)

### I. Format Invariants (deterministic tar.gz)
*VerifyBundle already implements this; clarify as spec.*
- **Tar Entry Order**: Dictionary order (deterministic).
- **Path Safety**: No absolute paths, no `..` traversal, no `NUL`, no `\` separators.
- **Type Constraints**: `dir`, `reg`, `symlink` only. Other Typeflags MUST FAIL.
- **Ownership/Permission Normalization**:
  - `uid`/`gid` = 0
  - `uname`/`gname` = "" (empty)
  - `dir` = 0755
  - `file` = 0644 or 0755 (others MUST FAIL)
- **Time Normalization**:
  - `mtime` precision = 1 second (`nsec`=0).
  - All entries MUST have identical `mtime`.
- **No PAX/xattr**: `LIBARCHIVE.*` or `SCHILY.xattr.*` headers MUST FAIL (leakage).
- **Gzip Header Invariants**:
  - `mtime` = contract epoch
  - `OS` byte = 255
  - `Name`/`Comment`/`Extra` = empty

### II. Layout Invariants (Required Files)
- **Mandatory**: `index`, `contract.json`, `SHA256SUMS`, `SHA256SUMS.seal`, `series.patch`.
- **Conditional**: `warnings.txt` required if `warnings_count > 0`.
- **Strict Mode**: `evidence/` directory content is REQUIRED.

### III. Manifest Invariants (Checksum & Seal)
- `SHA256SUMS.seal` MUST correctly verify `SHA256SUMS` (detect tampering).
- Entries in `SHA256SUMS` MUST exist and match the calculated SHA.
- **Exception**: Meta/Evidence files > 4MB are "excluded from analysis/hashing" (cannot be used for chain).

### IV. Evidence Binding Invariants (The King of S12-01 A)
- **Strict Requirement**: `evidence` existence is not enough. It MUST **bind** to the current commit.
- **Binding Rule**: Evidence content MUST contain the **40-char HEAD SHA** (matching `contract.head_sha`).
- **Chain Unification**:
  - `prverify` MUST output 40-char SHA (previously 12-char).
  - `reviewbundle` strict verification MUST fail if 40-char SHA is missing in evidence.

## Plan (pseudo-code)
try:
  1. Unify Chain:
     - prverify: `git rev-parse HEAD` (40-char)
     - reviewbundle: verify strict requires 40-char SHA in evidence
  2. Implement Invariants v1 Tests:
     - Add `cmd/reviewbundle/verify_*_test.go`
     - Test: strict + no evidence => E_EVIDENCE
     - Test: strict + evidence no SHA40 => E_EVIDENCE(binding)
     - Test: strict + evidence has SHA40 => PASS
     - Test: >4MB evidence => Unbindable => FAIL
catch:
  error: stop and document mismatch

## Stop Conditions
- Behavior change without tests => STOP
- New policy without a checker/gate => STOP
- Any path that can silently accept invalid input => STOP
