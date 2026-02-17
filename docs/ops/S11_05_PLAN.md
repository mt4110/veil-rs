# S11-05 PLAN — Reviewbundle Closeout & SOT/STATUS hygiene

## Goal
- S11-03/S11-04 の **Merged 後処理**を “壊れない儀式” に固定する
- STATUS.md を **100% / Last Updated / Evidence** まで正規化
- SOT 要件（check-sot）で二度と転ばない運用にする

## Non-Goals
- S11-03/S11-04 の機能差し戻し
- 仕様の大改造（必要なら S11-06 以降）

## Deliverables (files)
- docs/ops/S11_05_PLAN.md
- docs/ops/S11_05_TASK.md
- docs/ops/STATUS.md（S11-03/S11-04 を 100% に、S11-05 を 0% で追加/更新）
- docs/evidence/prverify/prverify_<NEW>.md（main HEAD で clean PASS の証拠）

## Invariants
- STATUS.md は **行順固定**（既存の順を崩さない）
- 変更は最小（% / Current / Last Updated / Evidence が主）
- PR は 1 本で S11-05 を Close まで運ぶ

## Execution (stopless)
- if repo not clean:
  - fix untracked/dirty を先に潰す（コミット or 破棄）
- run prverify on main HEAD (clean)
- copy report -> docs/evidence/prverify/
- update STATUS to point to the new evidence
- create SOT file that matches PR number (check-sot 対策)
- gates:
  - nix run .#prverify (PASS)
  - go test ./... (PASS)
