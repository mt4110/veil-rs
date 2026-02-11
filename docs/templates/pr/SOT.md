# PR-<NUM> — <slug> — SOT

## Context
- なぜ今やるか（背景/問題/非機能要件）
- Always Run 契約に従う（SOT / evidence / prverify / doc-links）

## Objective
- このPRの目的（1〜3行）

## Scope (In)
- 入れるもの

## Scope (Out)
- 入れないもの（将来PRへ）

## Plan
- docs/pr/PR-<NUM>-<slug>/implementation_plan.md を参照

## Always Run Evidence
- **Latest prverify report:** docs/evidence/prverify/prverify_<UTC>_<sha7>.md
- **Verification:** `nix run .#prverify` PASS
- **Doc-links:** docs に `file:` と `//` の生連結が無い
