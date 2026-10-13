# Testing Secrets Policy

## Guiding Principle
To prevent "False Positives" from GitHub Push Protection and other scanners, and to avoid confusion, **we aim to minimize hard-coded secret-looking strings in our test code.**

## Policy (v0.7.0+)

1.  **Runtime Generation**:
    Where possible, tests should generate fake secrets at runtime rather than embedding them as string literals.
    ```rust
    // Good
    let fake_key = format!("AKIA{}", "X".repeat(16));
    ```

2.  **Legacy Fixtures**:
    Some existing tests and fixtures contain hard-coded fake values. They are not automatically safe to publish or excluded from scans: exclusions depend on the effective configuration.
    The source root currently has no `veil.toml`; do not assume a repository configuration hides test values. Prefer generated fixtures or non-matching placeholders in new work.

3.  **No Real Secrets**:
    Under NO circumstances should a real, valid credential be committed to this repository, even for testing.
