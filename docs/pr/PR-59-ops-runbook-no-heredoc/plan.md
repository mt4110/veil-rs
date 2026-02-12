# PR59 plan (structured control words)

goal:
  - fix manual CLI operational hazards by freezing safe patterns into runbook
  - make closeout/evidence steps deterministic and degrade-model-safe

invariants:
  - docs-only (no code changes)
  - examples must NOT use heredoc
  - host side recommends: set -eo pipefail (NOT -u)
  - if -u is needed -> only inside bash -lc subshell
  - must reinitialize REPO/ISSUE/EVDIR per terminal session
  - skip requires 1-line reason; error stops immediately

paths:
  - runbook: docs/runbook/always-run.md
  - sot: docs/pr/PR-59-ops-runbook-no-heredoc.md
  - plan: docs/pr/PR-59-ops-runbook-no-heredoc/plan.md
  - task: docs/pr/PR-59-ops-runbook-no-heredoc/task.md
  - evidence_dir: docs/evidence/prverify/

flow:
  preflight:
    if repo_root_not_found -> error stop
    if runbook_missing -> error stop

  doc_change:
    edit runbook: add “No-Heredoc / StrictMode-safe manual CLI patterns”

  verification:
    run prverify -> must PASS
    locate latest prverify report for HEAD7 -> must exist
    copy report to docs/evidence/prverify -> must exist

  sot_update:
    pin Latest prverify report path to committed docs/evidence/prverify file

  finish:
    commit docs/*
    push branch
