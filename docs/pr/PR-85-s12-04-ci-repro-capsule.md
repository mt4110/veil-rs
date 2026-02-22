# PR-85 SOT — s12-04-ci-repro-capsule

## SOT
- PR: #85
- Repo: veil-rs
- Phase: S12-04 (CI Repro Capsule)
- Branch: HEAD
- Head: e484b6fff1f70590e42164a609f698f5f9d0fac4
- Board: docs/ops/STATUS.md
- Ops evidence: docs/evidence/ops/obs_20260222_s12-04.md

## What
- Add `prkit ci-repro` (stopless CI repro ritual capsule)
- Persist deterministic-ish artifacts: git probe / STATUS snapshot / fixed summary
- No kill-path: no os.Exit / panic / log.Fatal

## Verification
Local (representative):
- `nix develop -c go test ./...` : PASS (see logs under .local/obs)
- `nix run .#prverify` : PASS
- `reviewbundle create+verify (wip/strict)` : PASS (where available)

## Evidence
- WIP bundle: .local/review-bundles/veil-rs_review_wip_19800101_000000_e484b6fff1f7.tar.gz
- WIP sha256: a0589e278c7beebce87d625c54bed9cbdb1a6340dca315232a7a353ec5d72059
- STRICT bundle: .local/review-bundles/veil-rs_review_strict_19800101_000000_e484b6fff1f7.tar.gz
- STRICT sha256: 8fbdf80d86ee016625a8ef0163dd1484833669290a53186a40c937bebdb3fcbe
- prverify report: .local/prverify/prverify_20260221T233411Z_e484b6f.md

## Rollback
- Revert the merge commit for this PR:
  - `git revert <merge_commit_sha>`
