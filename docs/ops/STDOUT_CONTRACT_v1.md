# STDOUT Contract v1

**Canonical output format for all CLI tools in veil-rs.**

---

## Purpose

Every tool in veil-rs follows a single, machine-readable stdout contract.
This enables consumers (shell scripts, Go callers, CI pipelines) to parse
results **without relying on exit codes**.

---

## Output Line Types

| Prefix         | When                                    | Examples                                                 |
| -------------- | --------------------------------------- | -------------------------------------------------------- |
| `OK:`          | Successful operation or phase milestone | `OK: phase=end stop=0`, `OK: dir=obs count=3 newest=...` |
| `ERROR:`       | Failure condition (machine-readable)    | `ERROR: verify_failed path=x.tar.gz detail=...`          |
| `SKIP:`        | Operation intentionally skipped         | `SKIP: apply requires both --mode apply AND --apply`     |
| `WARN:`        | Warning, non-fatal                      | `WARN: dir=prverify size=600MB exceeds 500MB`            |
| `PASS:`        | All checks passed                       | `PASS: bundle verified path=...`                         |
| `FAIL:`        | All checks failed summary               | `FAIL: prverify components drift or fail.`               |
| `CANDIDATE:`   | GC candidate (localgc plan mode)        | `CANDIDATE: dir=obs name=x.txt age=8d ...`               |
| `INFO:`        | Informational (non-machine)             | `INFO: dry-run complete ...`                             |
| `## <section>` | Section header (human-readable)         | `## Plan`, `## Apply`                                    |

---

## The Stop Flag (Stopless Design)

**The `stop` flag is the single source of truth for consumer decisions.**

```
OK: phase=end stop=0   # All checks passed
OK: phase=end stop=1   # One or more checks failed
```

### Rules

1. **Always last line** — every tool's stdout MUST end with exactly one `OK: phase=end stop=<0|1>` line
2. **Monotonic** — once `stop=1` is set, it stays `stop=1`; no way to un-fail
3. **Process exit code is always 0** — consumers MUST NOT use exit codes for decision
4. **`ERROR:` implies `stop=1`** — if any `ERROR:` is emitted, `stop=1` must follow

---

## Consumer Contract

Any consumer of these tools MUST:

```go
// Correct: parse stop flag from stdout
if strings.Contains(out, "OK: phase=end stop=0") {
    return nil // success
}
if strings.Contains(out, "OK: phase=end stop=") {
    return fmt.Errorf("stop=1 (checks failed)")
}
// phase=end not found → something went wrong (process failure)
return fmt.Errorf("did not emit OK: phase=end (incomplete run)")
```

Any consumer MUST NOT:

```go
// Wrong: relying on exit code
if err := cmd.Run(); err != nil {  // FORBIDDEN
    return err
}
```

---

## Verified Entrypoints (as of S12-06B/C)

| Tool                | Binary                       | OK: phase=end? | Notes                  |
| ------------------- | ---------------------------- | -------------- | ---------------------- |
| prverify            | `cmd/prverify/main.go`       | ✓              | stop=0/1 from hasError |
| reviewbundle verify | `cmd/reviewbundle/main.go`   | ✓              | added S12-06B          |
| reviewbundle create | `cmd/reviewbundle/create.go` | ✓              | via CreateBundleUI     |
| localgc             | `cmd/localgc/main.go`        | ✓              | added S12-06C          |

## Verified Consumers (as of S12-05.6 / S12-06B)

| Consumer                   | File                                  | Parses stop=?             |
| -------------------------- | ------------------------------------- | ------------------------- |
| flake.nix preverifyScript  | `flake.nix` L196-199                  | ✓ grep -qF "stop=0"       |
| runPrverify (reviewbundle) | `cmd/reviewbundle/create.go` L332-336 | ✓ strings.Contains stop=0 |

---

## stderr Policy

- `stderr` is **informational only** (human UX)
- `stderr` contents are never machine-parsed
- Consumers MAY discard stderr
- `ERROR:` details may appear on BOTH stdout (canonical) and stderr (UX mirror)

---

## Invariants to Enforce

All tools must pass this audit:

```sh
# 1. No exit-code-based control flow in entrypoints
rg "os\.Exit|log\.Fatal|log\.Panic" cmd/ -g "*.go" | grep -v "main\.go:.*os\.Exit(run("

# 2. All entrypoints emit phase=end
for cmd in cmd/*/; do
  rg -l "OK: phase=end" "$cmd" | grep -v "_test" || echo "MISSING: $cmd"
done

# 3. No direct exit code consumer
rg "cmd\.Run\(\)" cmd/ -g "*.go" | grep -v "_test\|//\|FakeExec\|prverify/main"
```
