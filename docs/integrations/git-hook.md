# Native Git Hook Integration

If you don't use the `pre-commit` framework, you can use our shell script directly in your `.git/hooks`.

## Setup

1.  Copy the hook script:
    ```bash
    cp scripts/git-hooks/pre-commit.veil.sh .git/hooks/pre-commit
    ```
2.  Make it executable:
    ```bash
    chmod +x .git/hooks/pre-commit
    ```

## Functionality

The hook does the following:
1.  Checks if `veil` is installed (`cargo install veil`).
2.  Identifies files staged for commit (`git diff --cached`).
3.  Runs `veil scan` on those files with `--fail-on-findings`.
4.  If secrets are found, blocks the commit.

## Bypassing

If you need to commit despite findings (e.g. false positives you haven't whitelisted yet), use:

```bash
git commit --no-verify
```
