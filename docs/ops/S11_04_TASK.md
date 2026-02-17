# S11-04 Task: Hermetic Determinism Tests

## Phase S11-04 — Hermetic Determinism Tests

### C0 — Test Infrastructure
- [ ] Create `cmd/reviewbundle/testutil_gitrepo.go` (synthetic repo helper)
- [ ] Implement `InitRepo`, `CommitFile`, `Tag`, `MakeBranch` helpers

### C1 — Hermetic Migration
- [ ] Refactor `cmd/reviewbundle/determinism_test.go`
  - Remove dependency on `os.Getwd()` or external git repo
  - Use `testutil_gitrepo` to create temp repo in `t.TempDir()`
  - Verify deterministic output against known-good bundle or self-consistency
- [ ] Ensure `git format-patch` uses internal refs
- [ ] Verify `go test ./...` PASS locally

### C2 — Documentation
- [ ] Update `docs/ops/STATUS.md`
- [ ] Create Evidence `docs/evidence/prverify/prverify_...`
