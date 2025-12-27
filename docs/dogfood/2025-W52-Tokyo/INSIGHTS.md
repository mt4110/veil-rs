# System Insights (2025-W52)

## 1. SCORECARD_MISSING (High)
Scorecard CLI is missing from the execution environment.
Conclusion: Add `scorecard` to `flake.nix` runtimeInputs or workflow.

## 2. MISSING_ARTIFACT_LOG (Info)
Execution log `run.log` is missing.
Conclusion: Verify CI capture configuration in `dogfood_weekly.yml`.

## 3. UNEXPECTED_FAILURE_COUNT (Warn)
1 unexpected failure events recorded.
Conclusion: Investigate root cause in logs.

