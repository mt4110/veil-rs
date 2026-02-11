# PR55 — Pre-commit md file-url guard (no .sh)

## ✅ Summary
Land a **pre-commit guard** that blocks raw **`file:` immediately followed by `//`** in **staged Markdown (`.md`) files**,
**without adding any `*.sh` files**, so CI policy **No Shell Scripts** remains green.

This PR is intentionally minimal and deterministic:
- staged `.md` only
- fail-fast
- show offending lines (with line numbers)
- no auto-fix

---

## 🔗 Context
- PR54 merged ✅ (Issue #47 PR-B: bytes already 1.11.1 / rsa removed)
- Issue #47 remains OPEN (reopened) with progress comments recorded
- A local pre-commit guard was prototyped on branch `chore/add-pre-commit-hooks`
  but introduced `ops/cleanFormatter.sh`, which may violate CI policy.

---

## 🎯 Objective
Block commits when staged `.md` files contain raw **`file:` immediately followed by `//`**,
while keeping the repo free of newly added `.sh` files.

---

## ✅ Scope (In)
- Rename/remove/replace `ops/cleanFormatter.sh` so repo has **no new `.sh`**
- Keep `.githooks/pre-commit` as the entry point (no `.sh` extension)
- Maintain guard behavior:
  - staged `.md` only
  - fail-fast
  - print offending lines + line numbers

---

## 🚫 Scope (Out)
- Auto-fixing links (non-deterministic / meaning corruption risk)
- Broad refactors / unrelated formatting

---

## ✅ Always Run Contract
- plan/task:
  - `docs/pr/PR-55-pre-commit-file-url-guard-no-sh/implementation_plan.md`
  - `docs/pr/PR-55-pre-commit-file-url-guard-no-sh/task.md`
- verification:
  - `cargo test --workspace`
  - `nix run .#prverify` PASS
- evidence archived:
  - `docs/evidence/prverify/prverify_<UTC>_<sha7>.md`
- doc-links guard safe

---

## 🧩 Design Notes (Determinism & UX)
- Input must be **staged files only**:
  - `git diff --cached --name-only -- '*.md'`
- Detection:
  - use `PAT='file:'"//"` and detect `$PAT`
  - guard-safe split notation (like `file:` + `//`) is not matched
- Output:
  - print file name + `grep -n` output for exact offending lines
- Fail-fast:
  - stop on first offending file, but print all offending lines within that file

---

## 🛠️ Commands (Copy/Paste)

### 0) Move to repo root
```bash
cd "$(git rev-parse --show-toplevel)"
```

### 1) Switch branch

```bash
git switch chore/add-pre-commit-hooks
```

### 2) Remove `.sh` (rename only)

```bash
git mv ops/cleanFormatter.sh ops/cleanFormatter
perl -pi -e 's/\bops\/cleanFormatter\.sh\b/ops\/cleanFormatter/g' .githooks/pre-commit
chmod +x ops/cleanFormatter .githooks/pre-commit
rg -n "cleanFormatter\.sh" -S .githooks/pre-commit ops/cleanFormatter || true
git ls-files '*.sh' || true
```

### 3) Guard behavior tests

Negative test (must block):

```bash
tmp="docs/_tmp_fileurl_hook_test.md"
PAT='file:'"//"
printf '%s\n' "# test" "raw file url: ${PAT}example" > "$tmp"
git add "$tmp"
.githooks/pre-commit && echo "UNEXPECTED: passed" || echo "OK: blocked"
git restore --staged "$tmp"
rm -f "$tmp"
```

Positive test (must pass):

```bash
tmp="docs/_tmp_fileurl_hook_test_ok.md"
printf '%s\n' "# ok" "no raw file url here" > "$tmp"
git add "$tmp"
.githooks/pre-commit && echo "OK: passed" || (echo "UNEXPECTED: blocked"; exit 1)
git restore --staged "$tmp"
rm -f "$tmp"
```

### 4) Full verification

```bash
cargo test --workspace
nix run .#prverify
```

### 5) Archive evidence

```bash
UTC="$(date -u +%Y%m%dT%H%M%SZ)"
SHA7="$(git rev-parse --short=7 HEAD)"
mkdir -p docs/evidence/prverify
cp -a ".local/prverify/prverify_${UTC}_${SHA7}.md" "docs/evidence/prverify/prverify_${UTC}_${SHA7}.md"
```

### 6) Commit

```bash
git add ops/cleanFormatter .githooks/pre-commit docs/pr docs/evidence/prverify
git commit -m "chore: pre-commit md file-url guard (no .sh files)"
```

---

## 📌 SOT Commit

* SOT commit (HEAD short sha): `________`

---

## 🧾 Evidence

* Latest prverify report: `docs/evidence/prverify/prverify_20260211T094131Z_094edce.md`
* Notes:

  * Confirmed CI policy compliance: no new `*.sh`
  * Confirmed guard blocks raw `file:` immediately followed by `//` in staged `.md`

---

## ✅ Acceptance Checklist

* [ ] No tracked `*.sh` added (`git ls-files '*.sh'` unchanged / empty)
* [ ] pre-commit entrypoint remains `.githooks/pre-commit`
* [ ] staged `.md` containing raw `file:` immediately followed by `//` is blocked with line numbers
* [ ] `cargo test --workspace` PASS
* [ ] `nix run .#prverify` PASS
* [ ] evidence archived + linked above
* [ ] plan/task present and aligned
  