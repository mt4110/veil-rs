# PR58 — Issue #47 Closeout — plan (structured control words)

plan:
  goal:
    - close Issue #47 with short note + evidence pointers
    - confirm baseline "quiet main" with pointerable evidence (GitHub-visible)

  invariants:
    - evidence pointers must point to committed files (NOT .local/)
    - RunAlways: re-check and re-snapshot even if "already done"
    - no fake claims: if evidence capture fails => error stop

  paths:
    - plan: docs/pr/PR-58-issue-47-closeout.plan.md
    - task: docs/pr/PR-58-issue-47-closeout.task.md
    - sot : docs/pr/PR-58-issue-47-closeout.md
    - evidence_dir (committed): docs/pr/evidence/issue-47/

  preflight:
    if repo_root_not_found -> error stop
    if gh missing -> error stop
    if gh not authed -> error stop

  evidence:
    for item in [dependabot_open_alerts, prverify_quiet_main]:
      if capture failed -> error stop
      continue

  doc_update:
    if sot points to .local paths -> rewrite to docs/pr/evidence/issue-47 paths
    else -> skip

  issue_closeout:
    if Issue #47 is OPEN:
      comment -> close
    else if Issue #47 is CLOSED:
      comment -> skip close
    else:
      error stop

  pr_update:
    commit only docs/*
    if any .local staged -> error stop
