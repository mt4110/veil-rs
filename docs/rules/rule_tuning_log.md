# Rule Tuning Log

## v0.7.0 (2025-12-10)

### Dogfooding Analysis (v0.6.2)
- Scanned `veil-rs`, `rec-watch`, `veri-rs`.
- **False Positives**: 0 observed on actual source code.
- **Test Data**: 30+ findings in `veil-rs` pointed to `tests/` and `benches/`.
- **Action**:
    - No changes to core rules (`creds.*`, `jp.*`).
    - Added `veil.toml` to `veil-rs` root to ignore test directories:
        - `tests/fixtures`
        - `tests/data`
        - `docs/dogfood` (Report outputs)

### Configuration Validation
- Introduced `veil config check` to detect:
    - Dangerous Regex (ReDoS prevention for `(.+)+` etc).
    - Invalid config syntax.
