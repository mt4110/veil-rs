# Detect the license text in LICENSE.md

## SOT

- [OpenSSF Passing assessment for veil-rs](https://www.bestpractices.dev/en/projects/16556/passing)
- [Scorecard v5.6.0 license detection](https://github.com/ossf/scorecard/blob/v5.6.0/checks/raw/license.go#L109-L147)

## What

Place the existing Apache-2.0 text at the start of the root `LICENSE.md` and keep the dual-license notice and links after the text. The original `LICENSE-APACHE` and `LICENSE-MIT` files are the full license texts for distribution. The project continues to offer `MIT OR Apache-2.0`.

## Verification

- Compared the added Apache-2.0 text byte for byte with `LICENSE-APACHE` and verified that the original notice and its links are present.
- Ran the OpenSSF badge detector against the candidate. It reported `license_location = Met` for `LICENSE.md`.
- Licensee 10.1.0 matched `LICENSE.md` to `Apache-2.0` with 100% confidence.
- Scorecard v5.6.0 returned 9/10 for a simulated GitHub license response with `NOASSERTION` and 10/10 with `Apache-2.0`.

## Evidence

On `main` at `1a22289`, the published badge is Passing and Scorecard reports License 9/10. The production GitHub License API classification of the candidate and the resulting live Scorecard score remain unverified.
