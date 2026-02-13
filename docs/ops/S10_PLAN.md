# S10_PLAN — Next Epic Planning & Clean Rail

progress: 100%

## PLAN
IF repo_root_not_found THEN
  ERROR "must run inside git repo"
END

IF docs_dir_unknown THEN
  FOR candidate_dir IN ["docs/ops", "docs/pr", "docs/roadmap", "docs"] DO
    IF candidate_dir exists THEN
      SELECT docs_home = candidate_dir
      BREAK
    END
  END
  IF docs_home not set THEN
    ERROR "no docs directory found; create docs/ops or docs/"
  END
END

# --- Phase 0: Clean Rail baseline ---
IF working_tree_dirty THEN
  ERROR "do not proceed; stash or commit intentionally"
END

IF main_not_synced THEN
  ERROR "must fast-forward main first"
END

# --- Phase 1: Candidate discovery ---
FOR source IN ["open PRs", "open issues", "dependabot alerts", "roadmap files"] DO
  IF source available THEN
    CAPTURE evidence to docs_home/S10_evidence.md
  ELSE
    SKIP source
  END
END

# --- Phase 2: Scoring & selection ---
FOR each_candidate IN candidates DO
  SCORE impact/risk/effort/unblock/comparability
END

IF no_candidate_selected THEN
  SELECT fallback_epic = "clean rail improvement" OR "comparability improvement"
END

# --- Phase 3: Lock decision ---
WRITE selected_epic into docs_home/S10_SELECTED.md
WRITE acceptance_criteria into docs_home/S10_ACCEPTANCE.md

# --- Phase 4: Branch & PR rail ---
IF selected_epic requires code THEN
  DEFINE branch_name = "s10-00-pr-ritual-automation-v1"
  DEFINE PR series rules (small, evidence-first, contracts explicit)
ELSE
  DEFINE doc-only branch_name similarly
END

progress: 100%
