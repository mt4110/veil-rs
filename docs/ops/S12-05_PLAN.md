# S12-05 PLAN: ci-repro runner/DI alignment (v1)

## Goal
Align ci-repro to prkit runner + DI conventions, as cleanup-only (no feature expansion).

## Non-Goals
- No new user-facing features unless required for alignment
- No behavior changes except wiring/structure (must be evidenced)

## Deliverables
- ci-repro entry aligns to prkit runner contract
- ci-repro core uses injected deps (FS/Exec/Env/Clock/Log etc.)
- Tests + docs + STATUS.md updated with evidence

## Path Discovery (must be real paths)
- Use rg to locate:
  - ci-repro implementation files
  - prkit runner interface / DI container (or deps struct)
- Record discovered paths in evidence logs

## Steps (stopless)
1. Baseline capture
   - Run `go test ./...`
   - Run `nix run .#prverify` (if available)
   - Run current ci-repro representative command(s)
   - Save logs under `.local/obs/s12-05_*/`
   - If any baseline fails: mark ERROR and stop further refactor (do not exit non-zero)

2. Runner alignment
   - Refactor CLI to call prkit runner (or wrap runner under existing CLI)
   - Ensure runner receives context + deps in prkit style
   - Keep behavior stable

3. DI alignment
   - Identify direct side-effect usage (`os/exec`, `os.Getenv`, `time`, filesystem)
   - Introduce deps interface/struct consistent with prkit patterns
   - Thread deps through runner -> core
   - Add minimal unit tests with fake deps where valuable

4. Evidence + Docs
   - Update docs describing how to run ci-repro via prkit runner
   - Update STATUS.md (S12-05 row + Last Updated + Evidence pointer)

## DoD / Acceptance
- `go test ./...` PASS
- `nix run .#prverify` PASS (or documented SKIP with reason if not available in env)
- ci-repro representative run(s) produce expected outputs
- STATUS.md updated and consistent
