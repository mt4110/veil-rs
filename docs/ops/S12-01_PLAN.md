# S12-01 PLAN — strict evidence binding: allow local prverify evidence

## Goal
`go run ./cmd/reviewbundle create --mode strict` が、
- git repo を汚さず（cleanのまま）
- bundle 内 `review/evidence/` に **HEAD SHA を含む証拠（prverifyレポート）** を同梱でき、
- self-audit / verify を PASS できるようにする。

## Observed Failure (SOT)
- strict create:
  - E_EVIDENCE: binding no evidence file contains HEAD SHA ...
- strict create after copying report into docs/evidence:
  - E_CONTRACT: git repository is dirty (prohibited in strict mode)
- verify on a “dead tar”:
  - E_EVIDENCE ...

## Root Cause
- `create(strict)` が evidence を repo 内 `docs/evidence/prverify/` からのみ収集している。
- 最新 prverify レポートは `.local/prverify/` にあるため、bundle evidence に入らず E_EVIDENCE。
- repo 内へコピーすると untracked が増えて strict の clean 制約で E_CONTRACT。

## Contract (New / Canonical)
- strict create は evidence として次を扱う：
  1) repo evidence: `docs/evidence/prverify/*.md`（従来通り、履歴）
  2) local evidence: `.local/prverify/prverify_*.md` のうち **HEAD SHA を含む最新1件**
- local evidence を見つけられない場合は、bundle 作成を中断し、
  - operator 向けに “prverify を実行せよ” を明示する。
- self-audit 失敗時に “死体tar” を残さない：
  - `*.tmp` に書く → self-audit PASS → 最終ファイル名へ rename

## Non-goals
- verify 側の契約（「bundle内 evidence のどれかに HEAD SHA がある」）は変更しない。
- CI の設計全体を変えない（このフェーズでは strict evidence の供給経路だけ直す）。

## Implementation Outline (Pseudo)
- Inputs:
  - head = git rev-parse HEAD (full SHA)
- Find local prverify report:
  - for p in recent `.local/prverify/prverify_*.md` (newest first, limit N):
      - read file (replace errors)
      - if head in txt: select p; break
      - else continue
  - if not found:
      - print ERROR: no local prverify contains HEAD
      - stop (do not emit final tar)
- Bundle assembly:
  - include repo evidence dir as before
  - include selected local prverify as:
    - `review/evidence/prverify/<basename>`
- Atomic write:
  - write to `...tar.gz.tmp`
  - run self-audit against tmp
  - if PASS: rename tmp → final
  - else: keep tmp only in temp dir or remove it (policy)

## Stop Conditions
- Cannot locate code references for evidence collection / strict contract -> stop and record rg outputs.
- local prverify report not found for HEAD -> stop, instruct operator to run `nix run .#prverify`.
- self-audit fails -> stop and keep artifacts isolated under `.local/archive/...`.

## Evidence Strategy
- Evidence of fix:
  - unit tests for evidence selection + bundling
  - local run:
    - `nix run .#prverify` (to generate `.local/prverify/prverify_*_HEAD.md`)
    - `go run ./cmd/reviewbundle create --mode strict ...` (PASS)
    - `go run ./cmd/reviewbundle verify <bundle>` (PASS)
