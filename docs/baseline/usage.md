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
> This file contains fingerprints of your existing secrets. It classifies them as "Suppressed Debt" rather than "New Vulnerabilities."

### 2. Commit Baseline (Share it with CI)
Commit the baseline file to your repository so your CI system (and teammates) can use it.

```bash
git add veil.baseline.json
git commit -m "chore: add veil security baseline"
```
> [!TIP]
> **Security Note**: The baseline file contains hashed fingerprints and redacted snippets, not raw secrets. However, treat it with care as it reveals *where* your potential secrets are.

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

Veil uses a strict fingerprinting method (SHA256 of line content + context). This ensures that if you move a secret or change the surrounding code, it *may* reappear as "New." This is a **safe-by-design** feature to prevent you from accidentally exposing a secret you thought was suppressed.

## Output Formats

Veil adjusts its output based on the format to suit the audience:

| Format          | Content              | Purpose                                                                                              |
| :-------------- | :------------------- | :--------------------------------------------------------------------------------------------------- |
| **Text / JSON** | **New Only**         | Actionable feedback for developers. "Fix this *now*." Supressed findings are hidden to reduce noise. |
| **HTML**        | **New + Suppressed** | Full context for audit and review. Visually distinguishes new vs. suppressed findings.               |

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
> Baseline is strict. If you change the line number or content of a suppressed secret, it becomes "New" again. This is intentional. To fix, simply re-run `--write-baseline`.

## CI Example (GitHub Actions)

A minimal example for your `.github/workflows/security.yml`:

```yaml
jobs:
  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Veil
        run: curl -sSfL https://get.veil.sh | sh # (Replace with actual install method)
      
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
