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
    Existing test fixtures (e.g. `tests/fixtures/*.txt`) that contain hard-coded fake secrets (like `AKIA...EXAMPLE`) are currently **ignored** by configuration (`veil.toml`) to prevent self-detection.
    We will gradually migrate these to generated fixtures or non-matching patterns where feasible.

3.  **No Real Secrets**:
    Under NO circumstances should a real, valid credential be committed to this repository, even for testing.
