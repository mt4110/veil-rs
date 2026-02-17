# REVIEW BUNDLE CONTRACT v1.1

## 1. Mission & Philosophy
Review Bundle は「監査に耐える契約成果物 (contract artifact)」である。
作成ツール (go / bash) によらず、同一入力からは常に同一の bundle (byte-identical) が生成されなければならない。
また、verify コマンドは bundle の内容のみから正当性を証明できなければならない。

## 2. Canonical Structure (Layout)
Bundle MUST be a `.tar.gz` file containing the following structure:

- `review/INDEX.md` : Human-readable summary
- `review/meta/contract.json` : Machine-readable metadata (version, mode, base/head, epoch, counts)
- `review/meta/SHA256SUMS` : Checksums of all files (excluding itself)
- `review/meta/SHA256SUMS.sha256` : Checksum of SHA256SUMS
- `review/patch/series.patch` : Full patch from base to head
- `review/evidence/**` : Proof of verification (strict mode: required)
- `review/files/**` : Source file snapshots

## 3. Determinism Rules (MUST)
Output MUST be reproducible bit-for-bit given the same:
- Git references (HEAD, Base)
- Epoch timestamp (SOURCE_DATE_EPOCH or HEAD commit time)

### 3.1. Timestamp Normalization
- **Epoch Source**:
  - If `SOURCE_DATE_EPOCH` env var is set, use it.
  - Else, use git HEAD commit time (`%ct`).
- **Gzip Header**:
  - ModTime = Epoch (matches source)
  - OS = 255 (unknown)
  - Name/Comment = Empty
- **Tar Header**:
  - MTime = Epoch (matches source)
  - AccessTime / ChangeTime MUST be zero (epoch 0) or omitted.
  - PAX atime/ctime/mtime MUST NOT appear in the archive.

### 3.2. Entry Ordering
- All entries in the tarball MUST be sorted by their full path in **bytewise lexicographic order**.

### 3.3. Identity Anonymization (No Host Leak)
- **User/Group**:
  - UID = 0
  - GID = 0
  - Uname = "" (empty)
  - Gname = "" (empty)
- **Permissions (Mode)**:
  - Directory = `0755`
  - Regular File = `0644`
  - Executable Regular File = `0755`
  - Symlink = (Not validated; implementation dependent)
- **Extended Attributes / PAX**:
  - **PAX Records**: Only `path` and `linkpath` keys are permitted.
  - **Forbidden PAX Keys**: `mtime`, `atime`, `ctime` MUST NOT be present.
  - **Forbidden Xattrs**: `LIBARCHIVE.*`, `SCHILY.xattr.*`, or any other extended attributes MUST NOT be present.
  - Verification MUST fail if any forbidden or unknown keys are found.
- **Paths**:
  - Absolute paths MUST NOT be present.
  - `../` (parent traversal) MUST NOT be present.

## 4. Checksumming Rules (MUST)
Output MUST be verifiable via SHA256:

- **Regular File**: `sha256(file_content_bytes)`
- **Symlink**: `sha256("symlink\x00" + target_path)`
- **Directory**: (Not checksummed individually; included in manifest via layout if needed)

## 4. Evidence Binding
- **Strict Mode**:
  - Repository MUST be clean.
  - Evidence file (`docs/evidence/prverify/prverify_*.md`) bound to HEAD or ANCHOR MUST exist.
  - The evidence MUST be included in `review/evidence/`.
  - Fallback to "latest available" is PROHIBITED.
- **WIP Mode**:
  - Repository MAY be dirty.
  - Evidence MAY be missing.
  - If missing, a warning MUST be recorded in `review/meta/warnings.txt`.

## 5. Manifest & Integrity
- **SHA256SUMS**:
  - MUST list SHA256 hashes for all files in the bundle except `review/meta/SHA256SUMS` and `review/meta/SHA256SUMS.sha256`.
  - Format: `hash  path` (standard sha256sum format).
  - Path MUST be relative to the bundle root (e.g., `review/INDEX.md`).
- **SHA256SUMS.sha256**:
  - MUST contain the single SHA256 hash of `review/meta/SHA256SUMS`.
  - This "seals" the bundle manifest.
- **Symlink Checksums**:
  - For symlinks, the checksum in `SHA256SUMS` MUST be `sha256("symlink\x00" + target)`.

## 6. Verification Protocol (verify command)
A valid verify run involves:
1. **Structure Check**: Confirm required files exist.
2. **Leak Check**: Scan tar headers for non-zero UIDs, known xattr keys, or absolute paths.
3. **Integrity Check**:
   - Verify `review/meta/SHA256SUMS` against `review/meta/SHA256SUMS.sha256`.
   - Verify every file in the bundle matches the hash in `SHA256SUMS`.
4. **Contract Check**:
   - `contract.json` version matches tool capability.
   - If `mode=strict`, ensure evidence is present.

## 7. Versioning
- This document defines Contract Version: `v1.1`
