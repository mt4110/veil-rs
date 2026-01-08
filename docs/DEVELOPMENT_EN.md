# Developer Guide

This guide is for developers and contributors of `veil-rs`.
For the user manual, please refer to [README.md](../README_EN.md).

## Minimum Supported Rust Version (MSRV)
We support the latest stable Rust and **MSRV 1.82.0**.
- **Patch Policy**: Patch releases never bump MSRV.
- **Minor Policy**: Minor releases may bump MSRV (documented in release notes).

## Build from Source

To contribute or try the latest `main` branch:

```bash
git clone https://github.com/mt4110/veil-rs.git
cd veil-rs
```

### Nix (Recommended)

This project assumes a `nix develop` environment.
If your system Rust is old (e.g. <= 1.82.0), dependencies might fail to build.

```bash
# Enter development environment
nix develop

# Build locally
cargo build --release

# Or install to system (update veil command)
cargo install --path ./crates/veil-cli
```

> [!TIP]
> **Check MSRV (1.82.0)**
> ```bash
> nix develop .#msrv
> ```

### Without Nix
Requires Rust 1.82.0 or later.

```bash
cargo build --release
```

The binary is generated at `./target/release/veil`.

## Testing

veil-rs includes tests for secret detection rules (Slack, AWS, GitHub PATs, etc.).

To avoid GitHub Push Protection blocking pushes, we **never** hard-code real-looking secrets
as string literals. Instead, tests generate fake tokens at runtime via helper functions.

See [docs/TESTING_SECRETS.md](TESTING_SECRETS.md) for the full “Safety Contract”
and guidelines on adding new secret tests.
