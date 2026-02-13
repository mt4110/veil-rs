# S10-02 Task — pr-kit dry-run v1

> ルール: **この順番を崩さない**。  
> skip は理由を1行。error は即終了（嘘を付かない）。

## 0. Preflight (repo状態とパス確定)

- [ ] cd "$(git rev-parse --show-toplevel)"
- [ ] git status -sb
- [ ] Verify worktree is clean (`git status --porcelain=v1` is empty)
  - [ ] if not clean: error "worktree dirty => STOP (commit/stash first)"
- [ ] Verify repo layout (cmd/prverify exists)
  - [ ] if not found: error "prverify cmd not found => STOP (need repo layout output)"
- [ ] Verify repo layout (internal/ exists)
  - [ ] if not found: error "internal/ not found => STOP (do not invent new layout)"
- [ ] Verify repo layout (docs/ops exists)
  - [ ] if not found: error "docs/ops not found => STOP"

## 1. 実装ファイル作成（Goのみでロジック）

- [x] if exists("cmd/prkit"):
  - [x] error "cmd/prkit already exists => STOP"
- [x] create file: cmd/prkit/main.go
- [x] create file: internal/prkit/portable_evidence.go
- [x] create file: internal/prkit/run.go
- [x] create file: internal/prkit/tools.go
- [x] create file: internal/prkit/check_git.go

設計制約:
- [x] if any map[string]any を evidence 出力に使いそうなら:
  - [x] error "map order unstable => STOP (use struct + slice)"
- [x] if shell script にロジックを入れそうなら:
  - [x] error "logic must be Go => STOP"

## 2. CLI wiring (dry-run only)

- [x] pr-kit のフラグを実装
  - [x] if --dry-run が無い:
    - [x] exit_code=2 で FAIL evidence を出して終了
- [x] 実装するチェックは1個だけ: git_clean_worktree
  - [x] run: git status --porcelain=v1
  - [x] if output != "":
    - [x] FAIL evidence を出して exit_code=2
    - [x] STOP
  - [x] else:
    - [x] continue

## 3. portable evidence v1 出力（最優先）

- [x] evidence struct を確定（schema_version, timestamp_utc, mode, status, exit_code, git_sha, tool_versions, checks, command_list, artifact_hashes）
- [x] tool_versions 収集
  - [x] for tool in [go, git, rustc, cargo, nix]:
    - [x] if tool not found:
      - [x] skip "tool_versions:<tool>" reason="not found in PATH"
      - [x] continue
    - [x] else:
      - [x] capture version string
- [x] JSON 出力は struct の field order で固定
  - [x] if output is not stable/deterministic:
    - [x] error "portable json unstable => STOP"

## 4. Docs (ops) 追加

- [ ] create docs/ops/S10_02_PLAN.md（この設計を反映）
- [ ] create docs/ops/S10_02_TASK.md（このチェックリストを反映）
- [ ] append docs/ops/S10_evidence.md に evidence（コマンド + 出力）を追記
  - [ ] if docs/ops/S10_evidence.md が無い:
    - [ ] error "S10 evidence missing => STOP (unexpected repo state)"

## 5. Tests / Verification

- [ ] go test ./...
- [ ] go run ./cmd/prkit --dry-run
  - [ ] expect: PASS, exit_code 0
  - [ ] expect: checks[0].name == "git_clean_worktree"
  - [ ] expect: artifact_hashes == []

## 6. Commit / Push

- [ ] git status -sb（意図したファイルのみ増えてる）
- [ ] git add -A
- [ ] git commit -m "feat(prkit): add dry-run portable evidence v1"
- [ ] git push -u origin s10-02-pr-kit-dry-run-v1
