# Install an extracted CLI archive locally

Status: `reference`

[日本語](install.md)

The installer is included in [v0.22.2](https://github.com/mt4110/veil-rs/releases/tag/v0.22.2), v0.22.1, and v0.22.0. For v0.21.0 and rc.4, run the extracted CLI directly and compare its version with the original release.

## Verify the download

Run the block below in a new, empty directory. It needs curl, OpenSSH with SSH signature support, GitHub CLI (sign in with GH CLI if requested), and sha256sum or shasum. It defaults to Linux x86_64 musl; replace the first ZIP assignment with the commented assignment for macOS arm64. The checksum checks the selected ZIP against SHA256SUMS; the detached signature authenticates its bytes using the release key; the SLSA bundle ties those bytes to the specified GitHub Actions workflow and source commit. Compare the expected fingerprint with the [recorded release key fingerprint](https://github.com/mt4110/veil-rs/blob/7dbb3fa249c32315c33209b1ad87b00018857d5f/docs/ci/oss-cli-release.md#検証する) as obtained from a trusted checkout or channel.

```sh
(
  set -eu
  zip=veil-0.22.2-x86_64-unknown-linux-musl.zip
  # macOS arm64: zip=veil-0.22.2-aarch64-apple-darwin.zip
  repo=mt4110/veil-rs
  source_sha=7dbb3fa249c32315c33209b1ad87b00018857d5f
  release_url="https://github.com/$repo/releases/download/v0.22.2"
  for asset in "$zip" "$zip.sig" "$zip.intoto.jsonl" SHA256SUMS; do
    curl -fLO "$release_url/$asset"
  done

  awk -v zip="$zip" '$2 == zip {print; n++} END {if (n != 1) exit 1}' \
    SHA256SUMS > selected.sha256
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum -c selected.sha256
  else
    shasum -a 256 -c selected.sha256
  fi

  key=veil-rs-release-ed25519.pub
  curl -fLO "https://raw.githubusercontent.com/$repo/$source_sha/docs/security/$key"
  fingerprint=$(ssh-keygen -lf "$key" -E sha256 | awk 'NR == 1 {print $2}')
  test "$fingerprint" = 'SHA256:UdvtwJ2HTocMm2555crXrgGdHEd30wtRO7LZT65X7nE'
  awk 'NR == 1 {print "veil-rs-release namespaces=\"veil-rs-release\" " $1 " " $2}' \
    "$key" > allowed_signers
  ssh-keygen -Y verify -f allowed_signers -I veil-rs-release \
    -n veil-rs-release -s "$zip.sig" < "$zip"

  gh attestation verify "$zip" --bundle "$zip.intoto.jsonl" --repo "$repo" \
    --signer-workflow "$repo/.github/workflows/release_provenance.yml" \
    --source-ref refs/heads/main --source-digest "$source_sha" \
    --predicate-type https://slsa.dev/provenance/v1
)
```

If any check fails, stop before extracting or running the ZIP. After all checks pass, extract it into a new directory and keep the bundled license notices.

## Install

The macOS executable is ad-hoc signed. The ZIP signature does not provide Developer ID signing or notarization; Gatekeeper behavior after download remains unverified. Keep OS execution protection enabled.

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

Use the version reported by the bundled CLI. The current stable release reports `veil 0.22.2`. Older releases report their own version. Continue with the [first scan, staged check and next use](getting-started.en.md#scan-one-of-your-own-files). The installer and CLI select their help language separately; see the help-language guidance below.

## Conflicts, failures and returning to a previous version

Reinstalling the same version and content succeeds. A different packet at the same version path, an existing regular CLI file, or an unmanaged/damaged link is never intentionally replaced. Choose another prefix or decide locally how to handle the existing CLI.

To explicitly switch between versions installed under this script's layout:

```sh
sh install.sh --lang en --activate
```

Version ordering is not inferred; a downgrade is also explicit. Keep previous archives: run their `install.sh --activate` with the same prefix to switch back. Older 0.21.0 archives without an installer can still run directly from their original location; restore the previous PATH manually if appropriate. `--activate` does not force replacement of another tool's CLI.

Failures return exit code 2. An existing link is not switched before copying and verification finish. Failed `.install.*` staging directories or activation links may remain at the reported path for inspection. Retries use a new staging directory; old versions are not deleted, automatically uninstalled or cleaned up. After an activation failure, check command resolution and version again.

## CLI help language

Published 0.22.2, 0.22.1, and 0.22.0 support `veil --lang en --help` and `veil --lang ja help scan`. Published 0.21.0 / rc.4 binaries do not support this CLI option.

Priority: `--lang` > `VEIL_LANG` > `LC_ALL` > `LC_MESSAGES` > `LANG` > English. Empty environment values are ignored. Only the first nonempty locale is considered: a `ja` primary language selects Japanese; all other locales select English. Invalid explicit or `VEIL_LANG` values exit 2; a valid explicit option overrides an invalid environment value.

Across the 19 public help pages, `-h` summarizes the operation; `--help` and `help <command>` include examples and essential constraints. No arguments shows help in the same language. This does not translate scan results, JSON/SARIF or general runtime errors. The existing `init --wizard` language selection remains separate. The [English tutorial](getting-started.en.md) covers the first scan, staged checks and next use. Synthetic acceptance is separate from real development reuse.
