# PR-87 SOT — s12-05-5-prverify-speedup-v1

## SOT
- PR: #87
- Repo: veil-rs
- Phase: S12-05.5 (speed up local prverify)
- Branch: HEAD
- Head: 5eb8677d2db335f14ac7af030d125c8c75494b99
- Board: docs/ops/STATUS.md

## What
- スコープ制御 + キャッシュ固定 + 上限付き並列（任意）による、ローカル開発の `prverify` 高速化
- `prverify --mode local-fast` 追加
- `prverify --parallel N` 追加
- デフォルト挙動は変更なし（CI環境不変）

## Evidence
- prverify (full) pass report: `.local/prverify/prverify_20260223T154622Z_5fdeba2.md`
- prverify (local-fast) pass report: `.local/prverify/prverify_20260223T154836Z_5fdeba2.md`
- Ops OBS durations check: `.local/obs/s12-05-5_prverify_20260223T154625Z/step_durations.txt`

## Rollback
- Revert the merge commit for this PR:
  - `git revert <merge_commit_sha>`
