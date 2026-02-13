# S10-02 Task — pr-kit dry-run v1

> ルール: **この順番を崩さない**。  
> skip は理由を1行。error は即終了（嘘を付かない）。

## 0. Preflight (repo状態とパス確定)

- [x] cd "$(git rev-parse --show-toplevel)"
- [x] git status -sb
- [x] Verify worktree is clean (`git status --porcelain=v1` is empty)
  - [ ] if not clean: error "worktree dirty => STOP (commit/stash first)"
- [x] Verify repo layout (cmd/prverify exists)
  - [ ] if not found: error "prverify cmd not found => STOP (need repo layout output)"
- [x] Verify repo layout (internal/ exists)
  - [ ] if not found: error "internal/ not found => STOP (do not invent new layout)"
- [x] Verify repo layout (docs/ops exists)
  - [ ] if not found: error "docs/ops not found => STOP"

## 1. 実装ファイル作成（Goのみでロジック）

- [x] Verify repo layout (cmd/prkit exists)
  - [ ] if not found: error "cmd/prkit missing => STOP"
- [x] create file: cmd/prkit/main.go
- [x] create file: internal/prkit/portable_evidence.go
- [x] create file: internal/prkit/run.go
- [x] create file: internal/prkit/tools.go
- [x] create file: internal/prkit/check_git.go

設計制約:
- [x] if any map[string]any を evidence 出力に使いそうなら:
  - [ ] error "map order unstable => STOP (use struct + slice)"
- [x] if shell script にロジックを入れそうなら:
  - [ ] error "logic must be Go => STOP"

## 2. CLI wiring (dry-run only)

- [x] pr-kit のフラグを実装
  - [x] if --dry-run が無い:
    - Output FAIL evidence (JSON) and exit_code=2
- [x] 実装するチェックは1個だけ: git_clean_worktree
  - [x] run: git status --porcelain=v1
  - [x] if output == "":
    - Status PASS
  - [x] if output != "":
    - Status FAIL, exit_code=2

## 3. portable evidence v1 出力（最優先）

- [x] evidence struct を確定（schema_version, timestamp_utc, mode, status, exit_code, git_sha, tool_versions, checks, command_list, artifact_hashes）
- [x] tool_versions 収集
  - [x] for tool in [go, git, rustc, cargo, nix]:
    - [x] if tool not found:
      - [ ] skip "tool_versions:<tool>" reason="not found in PATH"
      - [ ] continue
    - [x] else:
      - [x] capture version string
- [x] JSON 出力は struct の field order で固定
  - [x] if output is not stable/deterministic:
    - [ ] error "portable json unstable => STOP"

## 4. Docs (ops) 追加

- [x] create docs/ops/S10_02_PLAN.md（この設計を反映）
- [x] create docs/ops/S10_02_TASK.md（このチェックリストを反映）
- [x] append docs/ops/S10_evidence.md に evidence（コマンド + 出力）を追記
  - [x] if docs/ops/S10_evidence.md が無い:
    - [ ] error "S10 evidence missing => STOP (unexpected repo state)"

## 5. Tests / Verification

- [x] go test ./...
- [x] go run ./cmd/prkit --dry-run
  - [x] expect: PASS, exit_code 0
  - [x] expect: checks[0].name == "git_clean_worktree"
  - [x] expect: artifact_hashes == []

## 6. Commit / Push

- [x] git status -sb（意図したファイルのみ増えてる）
- [x] git add -A
- [x] git commit -m "feat(prkit): add dry-run portable evidence v1"
- [x] git push -u origin s10-02-pr-kit-dry-run-v1

## 7. SOT (Post-PR)

- [x] cd "$(git rev-parse --show-toplevel)"
- [x] git status -sb
- [x] if docs/pr/ に新規 SOT が無い:
  - [x] veil sot new --epic A --slug prkit-dry-run-v1
- [x] edit: 生成された docs/pr/PR-*.md にガチガチ本文を貼る
- [x] git add docs/pr
- [x] git commit -m "docs(pr63): add SOT"
- [x] git push
- [ ] CIの “SOT Missing” が消えることを確認
