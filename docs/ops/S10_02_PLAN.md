# S10-02 Plan — pr-kit dry-run v1 (portable evidence format v1)

## Goal
- `pr-kit` は **まず `--dry-run` のみ**
  - “何を/どこに/どう作るか” を **出力するだけ**
  - **ファイルは触らない**
- **portable evidence format v1** を先に固定
  - exit_code
  - git_sha
  - tool_versions
  - command_list
  - status (PASS/FAIL)
  - artifact_hashes（dry-run なので基本 empty）
- shell は薄いラッパーのみ、ロジックは Go（Acceptance 準拠）
- **チェックは1個だけ**（気持ちよく次へ行くため）
  - `git status --porcelain=v1` が空であること（clean worktree）

## Non-goals (S10-02ではやらない)
- PR作成、ブランチ作成、commit、push、rebase
- 実際の prverify 実行（dry-run なので “予定” を出すだけ）
- ファイル生成（evidence の保存先を “提示” はするが書かない）

## Constraints
- 既存 repo 構成に合わせる（勝手な新階層を作らない）
- verify-only PASS を壊さない
- portable evidence は diff-friendly（順序安定、map を使わない）

---

## Path Pinning (実パス確定の方式)
> Note: 実装ディレクトリは `cmd/prkit` だが、ユーザー向けのコマンド名は `pr-kit` を予定（kebab-case）。

plan:
- for candidate in ["cmd/prverify", "cmd/prverify/main.go", "cmd/prverify/*.go"]:
  - if exists(candidate):
    - break
  - else:
    - continue
- if not found:
  - error "cmd/prverify not found => STOP (need ls cmd output)"

plan:
- if exists("internal"):
  - ok
- else:
  - error "internal/ not found => STOP (do not invent new layout)"

plan:
- if exists("docs/ops"):
  - ok
- else:
  - error "docs/ops not found => STOP"

plan:
- if exists("cmd/prkit"):
  - error "cmd/prkit already exists => STOP (avoid collision)"
- else:
  - ok (create new command in same family as prverify)

---

## CLI Contract (draft)
- command: `pr-kit`
- flags:
  - `--dry-run` (required in v1)
  - `--format portable-json` (default)
- exit:
  - PASS => exit_code=0
  - FAIL => exit_code!=0
- output:
  - portable evidence JSON のみ（最後に1発、安定フォーマット。余計な human text は出さない）

---

## Portable Evidence Format v1 (JSON, stable ordering)
(※ map を使わず struct + slice で出力順を固定する)

{
  "schema_version": 1,
  "timestamp_utc": "YYYYMMDDTHHMMSSZ",
  "mode": "dry-run",
  "status": "PASS|FAIL",
  "exit_code": 0,
  "git_sha": "…",
  "tool_versions": [
    {"name":"go","version":"go1.xx.x …"},
    {"name":"git","version":"git version …"},
    {"name":"rustc","version":"rustc …"},        // optional: if available
    {"name":"cargo","version":"cargo …"},        // optional: if available
    {"name":"nix","version":"nix …"}             // optional: if available
  ],
  "checks": [
    {"name":"git_clean_worktree","status":"PASS|FAIL","details":"…"}
  ],
  "command_list": [
    {"name":"git_status_porcelain","cmd":"git status --porcelain=v1"}
    // ここは“実行したコマンド”ではなく “v1で必須のチェックコマンド（予定含む）”
  ],
  "artifact_hashes": []
}

skip rules:
- if a tool is not installed:
  - skip "tool_versions:<name>" with 1-line reason
  - (e.g. "skip rustc: not found in PATH")
- skip は必ず理由を1行残す（未来の監査ログ）

---

## Implementation Sketch (pseudo)
plan:
- if flag --dry-run is missing:
  - error "v1 requires --dry-run => STOP"

plan:
- run check: git_clean_worktree
  - cmd: `git status --porcelain=v1`
  - if output != "":
    - status=FAIL
    - exit_code=2
    - print evidence
    - STOP
  - else:
    - continue

plan:
- collect git sha: `git rev-parse HEAD`
  - if fail:
    - error "cannot resolve git sha => STOP"

plan:
- collect tool_versions:
  - for tool in [go, git, rustc, cargo, nix]:
    - if tool exists:
      - run `<tool> version` or `--version`
    - else:
      - skip with reason (1 line)
      - continue

plan:
- build portable evidence object (struct)
- print portable evidence JSON (stable ordering)
- exit 0

---

## Evidence (S10_evidence.mdへ追記)
- `go run ./cmd/prkit --dry-run` の stdout を貼る
- 期待:
  - PASS
  - command_list に git_status_porcelain
  - checks に git_clean_worktree PASS
  - artifact_hashes empty
