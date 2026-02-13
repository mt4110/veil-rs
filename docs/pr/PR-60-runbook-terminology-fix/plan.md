# PR60 Plan (pseudocode)

IF `docs/runbook/always-run.md` に `bash -lc` を subshell と呼ぶ箇所がある THEN
  replace → “別 bash プロセス”
END

IF 用語混同を招く可能性がある THEN
  add Terminology Control section（別 bash プロセス / subshell（厳密））
END

DO: `rg -n "subshell" docs/runbook/always-run.md`
IF 定義セクション以外で subshell が残る THEN ERROR

DO: `nix run .#prverify` (PASS)
DO: `nix run .#check` (PASS)
