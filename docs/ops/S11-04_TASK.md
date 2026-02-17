# S11-04 TASK — Hermetic Determinism Test & CI Fixes

## Phase B — CI Fixes (Immediate)
- [x] B1: Apply changes to `.github/workflows/ci.yml`
  - [x] Replace `nix run .#go-test` with `nix develop -c go test ./...`
  - [x] Ensure `fetch-depth: 0` in `actions/checkout@v4`
- [ ] B2: Local verify (optional)
- [x] B3: Commit and Push CI fixes
- [ ] B4: Watch PR checks

## Phase C0 — RepoDir Injection
- [x] Locate `determinism_test.go` and `create.go`
- [x] Modify `CreateBundle` / `Contract` / Options to accept `RepoDir`
- [x] Ensure backward compatibility (default to current dir if empty)
- [x] Smoke test `TestCreate_Determinism`

## Phase C1 — Forge Hermetic Git Repo
- [x] Create `cmd/reviewbundle/hermetic_repo_test.go` (or add to existing if appropriate)
- [x] Implement `forgeHermeticRepo(t *testing.T) (dir string, baseRef string)`
- [x] Ensure fixed env vars (HOME, XDG_CONFIG_HOME, GIT_CONFIG_*, etc.)
- [x] Ensure fixed user/email
- [x] Create initial commit and a second commit

## Phase C2 — Hermetic Determinism Test
- [ ] Update `TestCreate_Determinism` in `cmd/reviewbundle/determinism_test.go`
- [ ] Use `forgeHermeticRepo`
- [ ] Run `CreateBundle` twice with fixed `SOURCE_DATE_EPOCH`
- [ ] Verify byte-identical output
- [ ] Verify generated bundle behaves correctly with `VerifyBundle`
- [ ] Verify `git format-patch` works correctly in the hermetic env

## Final
- [ ] Verify strictly `go test ./cmd/reviewbundle/...`
