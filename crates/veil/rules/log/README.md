# Veil Log Pack (OBS/SECRET/PII)

This pack is designed for log scrubbing.

## Placeholders (fixed)
- <REDACTED:OBSERVABILITY>
- <REDACTED:SECRET>
- <REDACTED:PII>

Do not introduce fine-grained placeholders (e.g., <REDACTED:JP:POSTAL>).
Use `tags` for detail instead.

## Policy
- `veil filter` should not fail on findings (exit 0).
- This pack masks the "map" (observability surface) as well as values.
