# v0.18.0 PR#1 — CI pin auto (Epic A)

## Goal
- `veil init --ci github` が手直し不要で tag-pinned を生成できる

## Scope
- CLI option: --pin-tag (auto/none/explicit)
- GitHub workflow template へ反映
- golden test 追加

## Acceptance Criteria
- stable版ではデフォで `--tag vX.Y.Z` が入る
- `--pin-tag none` で `--tag` を出さない
- `--pin-tag v1.2.3` で指定値を出す
- prerelease(例: 0.18.0-rc.1) の auto は安全側に倒す（後述）

## Worklog
- 2026-01-09: SOT created
