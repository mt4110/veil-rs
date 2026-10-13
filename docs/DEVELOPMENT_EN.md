# Developer Guide

This guide is for developers and contributors of `veil-rs`.
For the user manual, please refer to [README.md](../README_EN.md).

## Minimum Supported Rust Version (MSRV)
We support the latest stable Rust and **MSRV 1.92.0**.
- **Patch Policy**: Patch releases never bump MSRV.
- **Minor Policy**: Minor releases may bump MSRV (documented in release notes).

## Build from Source

To contribute or try the latest `main` branch:

```bash
git clone https://github.com/mt4110/veil-rs.git
cd veil-rs
```

### Nix (Recommended)

Use `nix develop` for the pinned development checks. The local CLI disables Git HTTPS/SSH transports and does not depend on OpenSSL.
For a standalone macOS candidate, use the native SDK and validate that no Nix/Homebrew dynamic dependencies remain. See [release checks](ci/oss-cli-release.md).

```bash
# Enter development environment
nix develop

# Build locally
cargo build --locked --release

# Or install to system (update veil command)
cargo install --locked --path ./crates/veil-cli
```

> [!TIP]
> **Check MSRV (1.92.0)**
> ```bash
> nix develop .#msrv
> ```

### Without Nix
Requires Rust 1.92.0 or later.

```bash
cargo build --locked --release
```

The binary is generated at `./target/release/veil`.

## Testing

veil-rs includes tests for secret detection rules (Slack, AWS, GitHub PATs, etc.).

To avoid GitHub Push Protection blocking pushes, new tests should generate fake tokens at runtime rather than embedding complete secret-looking literals.
Some legacy fixtures still contain fake literals; their exclusion depends on the effective scan configuration.

See [docs/TESTING_SECRETS.md](TESTING_SECRETS.md) for the full “Safety Contract”
and guidelines on adding new secret tests.

## Local OSS profile

Default `cargo build` / `cargo test` targets config, core, and CLI.
The existing Evidence Pack v1 verifier is part of default and minimal CLI checks; no governance feature is required.
Legacy service/UI/Guardian members require explicit package selection or `--workspace`.
Run `python3 scripts/check_architecture.py` before changing crate dependencies.

Release candidates are checked with `python3 scripts/check_cli_release.py --binary <binary>`.

## Contributing to the OSS CLI

Keep detection, rules, and masking in `veil-core`; argument parsing, output, and exit-code policy belong in `veil-cli`; configuration types and validation belong in `veil-config`.
Do not add network clients, license authentication, or services to the default dependency graph. Keep legacy work isolated by explicit package or feature selection. See [architecture](architecture/local-oss.md).

For a change, describe the trigger, expected result, failure behavior, and how it was verified. Use runtime-generated fake credentials, never real keys. Add regression coverage for changed detection, masking, baseline, or exit-code behavior.
Review dependencies for necessity, license, and security implications before adding them. Review generated reports and baseline metadata before publishing them.

Python 3.11+ is required for the repository's release checks. From the source root:

```bash
cargo fmt --all -- --check
cargo clippy --locked -p veil-cli -p veil-core -p veil-config --all-targets -- -D warnings
cargo test --locked
python3 scripts/check_docs_taxonomy.py
python3 scripts/check_architecture.py
python3 -B -m unittest discover -s scripts -p 'test_check_*.py'
```

Run minimal, JP feature and standalone LSP checks when the change affects feature boundaries; commands and platform requirements are in [release checks](ci/oss-cli-release.md).
Validate changed tutorial examples with the actual CLI, including nonzero exits; do not turn failures into success just to complete the walkthrough.
Before proposing a release, separate the intended public changes from existing work and private files. Follow the PR's SOT/verification requirements. This guide does not authorize publishing or merging.
