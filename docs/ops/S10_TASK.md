# S10_TASK — Next Epic Planning & Clean Rail

progress: 100%

## Task 0 — Resolve DOCS_HOME (deterministic)
- [x] DOCS_HOME=docs/ops を採用した事実を明記（“docs/ops exists” を根拠に1行）
- [x] SKIP: docs/pr（理由1行: docs/opsがあるため探索不要）
- [x] SKIP: docs/（理由1行: 同上）
- [ ] ERROR/STOP: docs/ops が無い場合（将来の監査向けの停止条件）

## Task 1 — Clean Rail verification
- [x] `git status --porcelain=v1` must be empty
- [x] `git switch main && git fetch -p && git pull --ff-only`
- [x] Run verification command(s) (prverify / cargo test / your standard)
- [x] Write evidence snippet into `${DOCS_HOME}/S10_evidence.md`

## Task 2 — Gather candidates
- [x] `gh pr list --state open --limit 50` -> append to evidence
- [x] `gh issue list --state open --limit 50` -> append to evidence
- [x] IF dependabot enabled:
  - [x] fetch open alerts -> append to evidence
- [x] IF roadmap files exist:
  - [x] read & extract candidate epics

## Task 3 — Score & select next epic
- [x] FOR each candidate:
  - [x] score Impact (1-5)
  - [x] score Risk (1-5)
  - [x] score Effort (1-5)
  - [x] mark Unblock YES/NO
  - [x] mark Comparability YES/NO
- [x] SELECT winner by rubric: **PR Ritual Automation**

## Task 4 — Lock decision & create branch
- [x] Write `${DOCS_HOME}/S10_SELECTED.md`
- [x] Write `${DOCS_HOME}/S10_ACCEPTANCE.md`
- [x] Decide branch slug: `s10-00-pr-ritual-automation-v1`
- [x] Create branch: `git switch -c s10-00-pr-ritual-automation-v1`
- [x] STOP before any code change: confirm invariants + acceptance criteria

progress: 100%
