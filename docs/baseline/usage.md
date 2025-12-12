# Baseline Scanning: "Stop the Bleeding"

Veil's **Baseline Scanning** allows you to adopt secret scanning in mature repositories without being blocked by existing technical debt.

Instead of demanding you fix hundreds of historical secrets immediately, Veil lets you **snapshot** the current state and fail CI only when **new** secrets are introduced. This philosophy is often called "Stop the Bleeding."

## Golden Path (3 Steps)

Adopt baseline scanning in less than 5 minutes:

### 1. Create Baseline (Snapshot current debt)
Run a scan and instruct Veil to write all current findings to a baseline file.

```bash
veil scan --write-baseline veil.baseline.json
```
> [!NOTE]
> This command exits with `0` even if findings are detected, as its purpose is to create a snapshot. The file categorizes existing findings as "Suppressed Debt".

### 2. Commit Baseline (Share it with CI)
Commit the baseline file to your repository so your CI system (and teammates) can use it.

```bash
git add veil.baseline.json
git commit -m "chore: add veil security baseline"
```
> [!TIP]
> **Security Note**: The baseline file contains hashed fingerprints and metadata (rule_id/path/line/severity), not raw secrets. However, it can reveal *where* potential secrets exist, so treat it as sensitive project data.

### 3. Run Scan in CI (Fail only on new leaks)
Update your CI pipeline to use the `--baseline` flag. Veil will now ignore the secrets listed in the baseline and report only fresh findings.

```bash
# In your CI script:
veil scan --baseline veil.baseline.json
```

---

## How it Works

When you run `veil scan --baseline <file>`:

1.  Veil detects secrets as usual.
2.  It compares each finding against the fingerprints in the baseline file.
3.  **Matches** are marked as **Suppressed**.
4.  **Non-matches** are marked as **New**.

Veil v1 uses a strict fingerprint: `SHA256(rule_id | path | line | masked_snippet)`. This favors safety over convenience—if you move a secret or change the surrounding code, it *may* reappear as "New" because the fingerprint changes. This is safe-by-design: it prevents you from accidentally re-exposing a secret you assumed was suppressed.

## Output Formats

Veil adjusts its output based on the format to suit the audience:

| Format          | Content              | Purpose                                                                                             |
| :-------------- | :------------------- | :-------------------------------------------------------------------------------------------------- |
| **Text / JSON** | **New Only**         | Actionable feedback for developers: fix this *now*. Suppressed findings are hidden to reduce noise. |
| **HTML**        | **New + Suppressed** | Full context for audit and review. Visually distinguishes new vs. suppressed findings.              |

## Exit Codes

Veil's exit codes are designed for CI stability:

*   **`0` (Success)**: No findings, OR only suppressed findings found. (CI Passes)
*   **`1` (Failure)**: One or more **NEW** findings detected. (CI Fails)
*   **`2` (Error)**: The baseline file is missing, corrupt, or invalid. (CI Fails Safely)

## When to Re-baseline

You should re-run `veil scan --write-baseline` when:

1.  **Code Refactoring**: You moved files or changed lines containing suppressed secrets, causing them to show up as "New."
2.  **Secret Rotation**: You rotated a secret and want to remove the old entry from legitimacy.
3.  **Periodic Cleanup**: You fixed several historical secrets and want to update the baseline to reflect the cleaner state.

> [!WARNING]
> Baseline is strict by design. If you change the line number or content of a suppressed secret, it becomes "New" again. To fix, simply re-run `--write-baseline`.

## CI Example (GitHub Actions)

A minimal example for your `.github/workflows/security.yml` using the install script:

```yaml
jobs:
  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Veil
        run: curl -fsSL https://raw.githubusercontent.com/mt4110/veil-rs/main/scripts/install.sh | sh
      
      - name: Veil Scan (Baseline)
        run: veil scan --baseline veil.baseline.json --format json
```

## FAQ

**Q: "No secrets found" vs "No new secrets found"?**
*   "No secrets found" means the repo is completely clean.
*   "No new secrets found" means secrets exist but all match the baseline (suppressed).

**Q: What if the baseline file is corrupted?**
Veil will exit with code `2` to prevent a false negative. Fix the JSON syntax or generate a new baseline.

**Q: Should I edit the baseline JSON manually?**
No. It is machine-generated. If you need to ignore a specific false positive permanently (not just suppress it as debt), use `.veilignore` or `veil.toml` rules instead.
