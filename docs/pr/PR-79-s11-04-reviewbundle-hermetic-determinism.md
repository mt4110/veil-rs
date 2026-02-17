## SOT
- Scope: S11-04 Reviewbundle — hermetic determinism
- PLAN: docs/ops/S11_04_PLAN.md
- TASK: docs/ops/S11_04_TASK.md

## What
- Hermeticなgit repo上で reviewbundle の determinism を保証するテストを追加
- contract（BaseRef=main / patch生成 / epoch / 並び順 / checksum）を固定化

## Verification
- nix develop -c go test -count=1 ./... (PASS)
- nix run .#prverify (PASS)

## Evidence
- prverify: docs/evidence/prverify/prverify_20260217T085024Z_12b08ca.md
