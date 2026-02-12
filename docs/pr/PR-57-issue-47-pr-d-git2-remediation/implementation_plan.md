# PR57 Implementation Plan v2 — Issue #47 PR-D (git2) Remediation
Ultra-sliced deterministic evidence loop (restart-friendly)

## Goal
Fix Dependabot `git2` alerts (GHSA-j39j-6gw9-jw6h) by ensuring `git2 >= 0.20.4` is resolved (target 0.20.4),
with deterministic evidence + Always Run Contract.

## Non-negotiable invariants
- worktree clean at start (or explicitly stashed)
- targeted update only (`cargo update -p ... --precise ...`)
- minimal diff
- every phase produces committed evidence (restart-safe)

## Branch
`feature/pr57-issue47-prd-git2-remediation`

## Evidence outputs (committed)
- docs/evidence/dependabot/pr57_git2_alerts_pre_<UTC>.json (+ .md)
- docs/evidence/dependabot/pr57_git2_alerts_post_<UTC>.json (+ .md)
- docs/evidence/deps/pr57_cargo_tree_git2_pre_<UTC>.txt
- docs/evidence/deps/pr57_cargo_tree_git2_post_<UTC>.txt
- docs/evidence/deps/pr57_cargolock_git2_pre_<UTC>.txt
- docs/evidence/deps/pr57_cargolock_git2_post_<UTC>.txt
- docs/evidence/prverify/prverify_<UTC>_<sha7>.md

## Restart-friendly protocol
At the end of every Phase:
1) write evidence files
2) `git status --porcelain=v1` must be explainable
3) commit with a phase-scoped message
If execution stops, re-run Phase boundary checks and continue.

---

# Phase 0 — Environment & footing (restart start)
0.0 Confirm repo root:
```bash
pwd
git rev-parse --show-toplevel
cd "$(git rev-parse --show-toplevel)"


0.1 Confirm git status:

git status --porcelain=v1


STOP if not clean. If dirty, stash:

git stash push -u -m "wip: before PR57 start"
git status --porcelain=v1


0.2 Confirm tools exist:

command -v gh
command -v jq
command -v rg
command -v cargo


STOP if missing.

0.3 Create branch:

git switch -c feature/pr57-issue47-prd-git2-remediation


0.4 Create directories:

mkdir -p docs/evidence/dependabot docs/evidence/deps docs/evidence/prverify
mkdir -p docs/pr/PR-57-issue-47-pr-d-git2-remediation


COMMIT (Phase0):

git add docs/evidence docs/pr/PR-57-issue-47-pr-d-git2-remediation
git commit -m "docs(pr57): scaffold evidence dirs and plan/task"

Phase 1 — PRE snapshots (deterministic)

1.0 Set UTC stamp:

UTC="$(date -u +%Y%m%dT%H%M%SZ)"
echo "$UTC"


1.1 Dependabot alerts snapshot (filter GHSA):

gh api "/repos/mt4110/veil-rs/dependabot/alerts?state=open" --paginate \
| jq '[.[] | select(.security_advisory.ghsa_id=="GHSA-j39j-6gw9-jw6h")]' \
| tee "docs/evidence/dependabot/pr57_git2_alerts_pre_${UTC}.json" >/dev/null

jq '.' "docs/evidence/dependabot/pr57_git2_alerts_pre_${UTC}.json" \
> "docs/evidence/dependabot/pr57_git2_alerts_pre_${UTC}.md"


1.2 Dependency graph snapshot:

cargo tree -i git2 | tee "docs/evidence/deps/pr57_cargo_tree_git2_pre_${UTC}.txt"


1.3 Cargo.lock proof:

rg -n 'name = "git2"|version = "' Cargo.lock \
| tee "docs/evidence/deps/pr57_cargolock_git2_pre_${UTC}.txt"


COMMIT (Phase1):

git add docs/evidence
git commit -m "evidence(pr57): pre snapshots (dependabot + cargo tree + lockfile)"

Phase 2 — Minimal upgrade (Cargo.toml + lockfile)

2.0 Inspect current dependency declaration:

rg -n 'git2\s*=' crates/veil-cli/Cargo.toml


2.1 Edit crates/veil-cli/Cargo.toml:

change ONLY the version to 0.20.4

preserve existing features exactly (if any)

2.2 Targeted lockfile update:

cargo update -p git2 --precise 0.20.4


If error mentions multiple versions:

run:

cargo tree -i git2


then try:

cargo update -p git2@0.19.0 --precise 0.20.4


(Use the actual old version shown.)

2.3 Quick compile gate:

cargo test -p veil-cli


COMMIT (Phase2):

git add crates/veil-cli/Cargo.toml Cargo.lock
git commit -m "deps(veil-cli): bump git2 to 0.20.4 (GHSA-j39j-6gw9-jw6h)"

Phase 3 — Fix breakages (only if tests fail)

3.0 Full tests:

cargo test --workspace


3.1 If failing:

Identify the minimal failing module/function

Apply smallest possible change to compile/test

No refactor, no cleanup

3.2 Re-run:

cargo test --workspace


COMMIT (Phase3, only if changes occurred):

git add -A
git commit -m "fix: adapt to git2 0.20 API (pr57)"

Phase 4 — POST snapshots

4.0 New UTC:

UTC_POST="$(date -u +%Y%m%dT%H%M%SZ)"
echo "$UTC_POST"


4.1 cargo tree post:

cargo tree -i git2 | tee "docs/evidence/deps/pr57_cargo_tree_git2_post_${UTC_POST}.txt"


4.2 lockfile post:

rg -n 'name = "git2"|version = "' Cargo.lock \
| tee "docs/evidence/deps/pr57_cargolock_git2_post_${UTC_POST}.txt"


4.3 dependabot post snapshot:

gh api "/repos/mt4110/veil-rs/dependabot/alerts?state=open" --paginate \
| jq '[.[] | select(.security_advisory.ghsa_id=="GHSA-j39j-6gw9-jw6h")]' \
| tee "docs/evidence/dependabot/pr57_git2_alerts_post_${UTC_POST}.json" >/dev/null

jq '.' "docs/evidence/dependabot/pr57_git2_alerts_post_${UTC_POST}.json" \
> "docs/evidence/dependabot/pr57_git2_alerts_post_${UTC_POST}.md"


COMMIT (Phase4):

git add docs/evidence
git commit -m "evidence(pr57): post snapshots"

Phase 5 — Always Run Contract (prverify + evidence)

5.0 Run prverify:

nix run .#prverify


5.1 Ensure prverify evidence exists under docs/evidence/prverify/.
If prverify wrote to .local/..., copy it:

# example (adjust path to actual output)
cp -a .local/prverify/prverify_*.md "docs/evidence/prverify/"


5.2 Record sha7:

SHA7="$(git rev-parse --short=7 HEAD)"
echo "$SHA7"


COMMIT (Phase5):

git add docs/evidence/prverify
git commit -m "evidence(pr57): prverify report"

Phase 6 — Cockpit check

6.0 Discover cockpit command (repo-specific):

rg -n "cockpit" -S .github ops docs || true


6.1 Run the discovered check command and confirm PASS.
If multiple candidates exist, pick the one used in recent PR logs.

6.2 Summarize the command + PASS output snippet into SOT.

Phase 7 — SOT doc

7.0 Create docs/pr/PR-57-issue-47-pr-d-git2-remediation.md with:

objective + why Issue #47 was premature closure

what changed (git2 version, lockfile, any minimal code fix)

Always Run Contract results

links to all evidence filenames (pre/post/prverify)

note: dependabot closure may lag behind lockfile resolution

COMMIT (Phase7):

git add docs/pr
git commit -m "docs(pr57): SOT with evidence links"

Phase 8 — Final re-run + push

8.0 Re-run:

cargo test --workspace
nix run .#prverify


8.1 Push:

git push -u origin feature/pr57-issue47-prd-git2-remediation


Done.
