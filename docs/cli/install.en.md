# Install an extracted CLI archive locally

Status: `reference`

[日本語](install.md)

This guide applies to published [0.22.1](https://github.com/mt4110/veil-rs/releases/tag/v0.22.1) and 0.22.0 ZIPs containing `install.sh`. Published 0.21.0 and rc.4 ZIPs lack the installer and remain unchanged; run `./veil` directly from their extracted directory and compare its version with the original Release.

## Install

Supported installation platforms are Linux x86_64 and macOS ARM64. Linux distribution additionally requires verified static musl linkage before release. macOS remains an ad-hoc-signed trial asset: Developer ID signing, notarization and downloaded Gatekeeper behavior are unverified. This guide does not disable OS execution protection.

Check the release source and outer SHA256SUMS before extracting into a new directory. Checksums alone do not authenticate the publisher. Where needed, verify the signed tag with a trusted public key and compare its ZIP hashes.

From the extracted directory, run:

```sh
sh install.sh --lang en
```

The installer uses `sh`, `awk`, `sha256sum` or `shasum`, and ordinary OS tools such as `cp`, `mv` and `ln`. It needs no Rust, sudo, Python or network. It verifies internal checksums, OS/CPU and the CLI version before copying, then verifies the copied packet. Symbolic links and unsafe internal paths are rejected.

By default, the complete packet is installed at `~/.local/share/veil/<version>/<platform>/`, with a `~/.local/bin/veil` symlink. The installation platform is `x86_64-linux` or `aarch64-darwin`; it is not a Rust ABI target triple. Documents, license originals, third-party notices and required source archives remain alongside the CLI.

Choose another absolute prefix when necessary:

```sh
sh install.sh --prefix "$HOME/veil-tools" --lang en
```

A prefix must not contain symlinks, `.`/`..` components, newlines or the PATH separator `:`. Spaces and single quotes are supported. Relative prefixes and `/` are rejected.

## Set PATH once

The script prints a PATH line but does not edit `.zshrc`, `.zprofile`, `.bashrc`, `.bash_profile` or `.profile`. Use the line for your chosen prefix. For the default prefix:

```sh
export PATH="$HOME/.local/bin:$PATH"
```

- Interactive zsh: add it once to the active `~/.zshrc`.
- Interactive non-login bash: add it to `~/.bashrc`.
- Login bash: use the active `~/.bash_profile`, `~/.bash_login` or `~/.profile`. If that profile already sources `.bashrc`, do not duplicate the same line.

No extra line is needed if the bin directory is already in PATH. Open a new terminal and verify both the resolved path and version. If another CLI is selected, inspect existing commands and PATH order.

```sh
command -v veil
veil --version
```

Use the version reported by the bundled CLI. The current stable release reports `veil 0.22.1`. Older releases report their own version. Continue with the [first scan, staged check and next use](getting-started.en.md#scan-one-of-your-own-files). The installer and CLI select their help language separately; see the help-language guidance below.

## Conflicts, failures and returning to a previous version

Reinstalling the same version and content succeeds. A different packet at the same version path, an existing regular CLI file, or an unmanaged/damaged link is never intentionally replaced. Choose another prefix or decide locally how to handle the existing CLI.

To explicitly switch between versions installed under this script's layout:

```sh
sh install.sh --lang en --activate
```

Version ordering is not inferred; a downgrade is also explicit. Keep previous archives: run their `install.sh --activate` with the same prefix to switch back. Older 0.21.0 archives without an installer can still run directly from their original location; restore the previous PATH manually if appropriate. `--activate` does not force replacement of another tool's CLI.

Failures return exit code 2. An existing link is not switched before copying and verification finish. Failed `.install.*` staging directories or activation links may remain at the reported path for inspection. Retries use a new staging directory; old versions are not deleted, automatically uninstalled or cleaned up. After an activation failure, check command resolution and version again.

## CLI help language

Published 0.22.1 and 0.22.0 support `veil --lang en --help` and `veil --lang ja help scan`. Published 0.21.0 / rc.4 binaries do not support this CLI option.

Priority: `--lang` > `VEIL_LANG` > `LC_ALL` > `LC_MESSAGES` > `LANG` > English. Empty environment values are ignored. Only the first nonempty locale is considered: a `ja` primary language selects Japanese; all other locales select English. Invalid explicit or `VEIL_LANG` values exit 2; a valid explicit option overrides an invalid environment value.

Across the 19 public help pages, `-h` summarizes the operation; `--help` and `help <command>` include examples and essential constraints. No arguments shows help in the same language. This does not translate scan results, JSON/SARIF or general runtime errors. The existing `init --wizard` language selection remains separate. The [English tutorial](getting-started.en.md) covers the first scan, staged checks and next use. Synthetic acceptance is separate from real development reuse.
