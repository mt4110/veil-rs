# OpenSSF license discovery

## SOT

- [OpenSSF Passing criteria for veil-rs](https://www.bestpractices.dev/en/projects/16556/passing)
- [OpenSSF's license file detection rules](https://github.com/ossf/best-practices-badge/blob/9a7839f4e85a9c51cde24efa5cf386f97c2f2921/app/lib/repo_files_examine_detective.rb#L151-L165)

## What

Add a root `LICENSE.md` that states the existing dual-license choice and links to the full license texts. The OpenSSF detector recognizes `LICENSE.md`; its filename pattern misses `LICENSE-MIT` and `LICENSE-APACHE`.

## Verification

- Replay the OpenSSF filename and size rule on the proposed root files.
- Check that both links resolve to the existing license files and that the SPDX expression matches `Cargo.toml`.
- After merge, resave the badge assessment and verify the live Passing badge and Scorecard result.

## Evidence

Before this change, the live badge reported 99%, and Scorecard v5.6.0 scored `CII-Best-Practices` 2/10 with `badge detected: InProgress`.
