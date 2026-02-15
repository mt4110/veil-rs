# SOT: PR-TBD-v1-epic-A-s10-09-prkit-exec-hardening

## Goal
prkit の外部コマンド実行を「shell無し・契約明確・portable evidence」を満たす形に harden する。
特に argv/cwd/env/stdout/stderr/exit を決定論で記録し、テストは 実プロセスを起動しない。

## Non-goals
- テスト実行で 実 git / 実プロセスを起動しない（FakeRunner で完結）
- portable evidence に repo絶対パスを混入させない（argv だけでなく stdout/stderr も対象）
- stdout/stderr を混ぜて “再解釈” しない（証拠はツール出力を尊重）

## Changes
- **ExecRunner 抽象を中核に据え、Prod/Fake を分離**
- **command_list（portable evidence）に 実行契約を明示:**
  - argv / cwd_rel / exit_code / stdout / stderr / error_kind
  - env の継承有無を env_mode、effective env の識別子を env_hash で記録
- **review_bundle 実行の portability hardening:**
  - scriptPath は相対パス
  - stdout のみから OK 行を抽出（stderr 混合パース撤去）
  - RepoRoot 相対化により stdout/stderr への絶対パス混入を止血
- **resolveDir を Join+Clean 化し、repo外脱出を禁止**
- **TimeoutMs を実装（WithTimeout）**

## Evidence
- Evidence: `docs/evidence/prverify/prverify_20260215T012259Z_e70ca0f.md`

## Verification (Local/Hermetic)
- Hermetic gate: `nix run .#prverify` (PASS)

## Risk / Mitigation
- **env 記録の肥大化** → full env を保存せず env_hash で識別（差分は env に保持）
- **既存出力に絶対パスが混じるリスク** → RepoRoot 相対化（stdout/stderr 正規化）

## Rollback
prkit の evidence schema 変更が問題になる場合:
- env_mode/env_hash を optional として扱い、旧フィールド互換を維持
