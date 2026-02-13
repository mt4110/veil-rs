# S10_ACCEPTANCE — PR Ritual Automation (Phase 1)

## Goal
Human judgement is reserved for "Merge" and "Strategy"; all ritual steps (creation, verification, archival) are driven by verifiable scripts.

## Success Criteria (v1)
1. **Automation Kit established**: A script or tool (e.g. `ops/pr-kit`) can:
    - Generate SOT/Plan/Task from templates.
    - Run `prverify` and capture evidence with SHA7/UTC naming.
    - Update SOT with latest evidence link.
2. **Deterministic Evidence (Portable-first)**: Portable evidence は 同一コード状態なら bit-identical（timestamps等の可変要素は出さない）。Raw evidence は監査の真実として保持するが、コンパイル時間/実行時間などの揺れは許容する。
    - RAW: フルログ（コンパイル/時間/詳細を含む）
    - PORTABLE: exit_code, git_sha, tool_versions, command_list, PASS/FAIL, artifact_hashes の固定フォーマット
3. **STOP-heavy Safety**: If any step (lint, check, test, evidence) fails, the process aborts immediately with a clear recovery hint.
4. **Shell Policy (Go-first, thin wrappers allowed)**: 新規ロジックは Go（cmd/）に寄せる。Shell は 薄いラッパー/互換用途のみ許容（Goバイナリを呼ぶだけ、判断ロジックを持たない）。CI の主導権は Go/Nix/Make のレールを優先する。

## Verification
- Run the kit against a dummy PR branch.
- Confirm `docs/pr` and `docs/evidence` are correctly populated.
- Confirm `prverify` PASS on the resulting tree.
