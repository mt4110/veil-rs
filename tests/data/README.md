# Test Data Directory

This directory contains **official test fixtures** used by `veil-rs` integration and unit tests.

## Naming Convention

To keep this directory organized, please follow these naming conventions for new files:

- **Valid Secrets (Must Match)**: `*_hit.txt`
  - Example: `creds_aws_access_key_hit.txt`
  - Contains strings that *should* trigger a finding.

- **False Positives (Must NOT Match)**: `*_fp.txt`
  - Example: `creds_aws_access_key_fp.txt`
  - Contains strings that look like secrets but *should not* trigger a finding (e.g. example values, unrelated hashes).

## Grouping (Future)

If the number of files grows significantly (>50), we will reorganize them into subdirectories based on category:

- `tests/data/creds/*.txt`
- `tests/data/pii/*.txt`
- `tests/data/other/*.txt`

For now, a flat structure is sufficient.

## Legacy / Unused

Old testing artifacts that are kept for reference but not actively used in the main test suite are located in `dev/legacy/`.
