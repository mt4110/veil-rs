# PR-<NUM> task — <slug>（Always Run）

## 0) Preconditions
```bash
cd "$(git rev-parse --show-toplevel)"
git status --porcelain=v1 && test -z "$(git status --porcelain=v1)"
```

## 1) Implement
- [ ] （具体作業）

## 2) Sanity Check
```bash
rg -n "file:/{2}" docs || true
```

## 3) Verify
```bash
nix run .#prverify
```

## 4) Evidence
```bash
SHA7="$(git rev-parse --short=7 HEAD)"
UTC="$(date -u +%Y%m%dT%H%M%SZ)"
mkdir -p docs/evidence/prverify
SRC="$(find .local -maxdepth 3 -type f -name "prverify_*_${SHA7}.md" 2>/dev/null | sort -r | head -n 1)"
test -n "$SRC"
cp -a "$SRC" "docs/evidence/prverify/prverify_${UTC}_${SHA7}.md"
```

## 5) Update SOT
- [ ] Update `Latest prverify report` path in SOT.

## 6) Commit / Push
```bash
git add .
git commit -m "<type>(pr<NUM>): <summary>"
git push -u origin <branch>
```
