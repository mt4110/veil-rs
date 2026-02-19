
if repo_root is empty:
  print("ERROR: not in git repo")
  STOP

BR = current branch
if BR != "s12-03-strict-capsule-v1":
  print("WARN: branch mismatch: " + BR)
  continue

# --- Fix-1: SOT missing ---
if no file exists under docs/pr/*.md in this branch:
  PRNUM = try get from gh (else "TBD")
  slug  = sanitized(BR)
  create docs/pr/PR-{PRNUM}-{slug}.md with standard sections
  git add docs/pr/...
  git commit message "docs(pr): add SOT for PR-{PRNUM} ({slug})"
else:
  print("SKIP: SOT already exists")

# --- Fix-2: Go test failing due to git identity ---
# Option A (robust): patch mustRunGit helper to set GIT_AUTHOR/COMMITTER env
# Option B (minimal): set git config user.name/email in capsule_test.go before first commit

if patch applied:
  run go test ./...
  if output contains FAIL:
    print("ERROR: go test failed; see log")
    STOP

# --- (Optional) Copilot “correctness debt” cleanup ---
# - os.WriteFile error handling
# - STATUS Last Updated pointer truth
# - obs evidence file include outputs
# - strict evidence binding: unify 4MB rule in resolveEvidence

git push
wait CI green
