# JP PII Fixture Policy

JP PII fixtures are contract data for deterministic detection behavior. Keep the
sets balanced: every new positive class must add at least one matching negative
class that looks similar enough to catch false positives.

## Layout

```text
tests/fixtures/jp_pii/positive/
tests/fixtures/jp_pii/negative/
```

## Positive Fixtures

Positive fixtures must contain realistic formatting variance that should be
detected after JP normalization, such as fullwidth digits, fullwidth spaces, and
JP hyphen variants. Do not include real personal data.

## Negative Fixtures

Negative fixtures must cover adjacent non-PII formats: build numbers, order
numbers, version numbers, sample/test payment numbers, and dummy identifiers.
Prefer examples that share separators or digit counts with the positive case.

## Assertions

Scanner tests must assert both detection outcome and span behavior:

- Matches are extracted from normalized text.
- Masking uses original byte spans.
- Editor ranges use UTF-16 positions.
- Raw matched content is not introduced to Local API or Evidence fixtures.
