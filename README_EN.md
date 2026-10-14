# Veil — Local secret detection and masking

0.21.0-rc.2 is a draft candidate containing the main fixes for output masking, input/configuration errors, and safe file rewriting. It is not publicly released. Public download examples still point to rc.1; for a local rc.2 archive, confirm `veil --version` reports `0.21.0-rc.2`. The macOS candidate is ad-hoc signed, without Developer ID signing or notarization.

The 0.21.0-rc.1 trial is limited to Linux x86_64 musl and macOS ARM64. The macOS binary is ad-hoc signed; Developer ID signing, notarization, and Gatekeeper behavior after download remain unverified. Windows, Intel Mac, Linux ARM64, and standalone LSP distribution are excluded. Windows candidate notes below describe historical checks.

2026-10-13: The 0.21.0-rc.1 candidate fixes finding-limit completeness being lost after baseline suppression and the first Git commit being left unscanned. The corrected Linux x86_64 static musl candidate passed native build, startup and distribution checks on Ubuntu 24.04.5 LTS (Linux 7.0.0-38-generic). Older ARM64, QEMU and Windows results do not validate this candidate; native Windows/MSVC remains unverified. Additional fixes for interactive completeness and missing Git blobs passed regressions on macOS and native Linux tests and distribution revalidation. See the [release checklist](docs/ci/oss-cli-release.md).

Veil is an open-source Rust CLI for local secret detection and masking in source code and logs.
The default CLI includes no external API client, license authentication, or telemetry sender. Built-in rules work without initialization.
Detection can produce false positives and miss secrets; it does not prove that a project is secret-free.

[日本語](README.md) · [Tutorial (日本語)](docs/cli/getting-started.md) · [Usage guidelines (日本語)](docs/cli/usage-guidelines.md) · [CLI](docs/cli/README.md) · [Development](docs/DEVELOPMENT_EN.md) · [Architecture](docs/architecture/local-oss.md)

## Install and scan

### Binary archive

Extract the archive; it contains `veil` (`veil.exe` on Windows), both READMEs, `docs/`, project license texts, and `SHA256SUMS`.
On macOS/Linux, run the following from the extracted directory. The PATH change applies to the current shell.

```bash
./veil --version
export PATH="$PWD:$PATH"
veil --help
```

On Windows PowerShell, check `./veil.exe --version`, then use the binary's full path or add its directory to PATH.
The [tutorial](docs/cli/getting-started.md) and [usage guidelines](docs/cli/usage-guidelines.md) are included for offline reading.
Source code and development scripts are not bundled; development commands require a source checkout.
The local candidate is preparation material, not evidence of publication or validation on every OS.

### Linux environments and requirements

The preferred Linux distribution candidate uses static musl linking. The 0.21.0-rc.1 x86_64 candidate passed native build, startup, scans, masking and exit codes 0/1/2 on an x86_64 machine running Ubuntu 24.04.5 LTS (Linux 7.0.0-38-generic).
ELF checks found no PT_INTERP, DT_NEEDED or GLIBC references. Users need no installed Rust, shared libc, libz or libgcc runtime. License originals, required source archives, checksums and bundled links were checked.
The musl candidate including the interactive/Git-read fixes passed 357 default, 356 minimal, 358 JP and 25 LSP tests, plus 84 related release-profile regressions. In 21 direct cases, the CLI extracted from the ZIP returned exit code 2 for incomplete scans, emitted no partial report, refused new baselines and preserved existing baselines. LSP stdio startup passed through both the CLI and the standalone server. These runs overlap and must not be added as distinct test cases.
Older ARM64 and QEMU x86_64 results do not validate this RC on ARM64 or other distributions.
The minimum kernel version is not established. Older kernels, other distributions, separate machines/VMs, other CPUs and performance remain untested.

The previous Debian 13 GNU candidate needs glibc 2.39 or later plus libz.so.1 and libgcc_s.so.1; it failed to start on Ubuntu 22.04 with glibc 2.35.
Choose the correct CPU and distribution variant. See the [Linux distribution guide (日本語)](docs/cli/linux-distribution.md) for build instructions, verification and limits.

### Windows requirements and validation scope

The x86_64 Windows GNU candidate was cross-built with Rust 1.92.0 / MinGW-w64 and passed startup, masked JSON and exit codes 0/1/2 under Wine 10.0 plus QEMU on an actual ARM64 host. The target baseline is Windows 10+ / Server 2016+; native Windows and the existing MSVC CI candidate remain untested.
Declared normal/delay PE imports contain 12 Windows-provided dependencies and no additional MinGW runtime DLLs. Static MinGW/GCC runtime notices are included. Clean Windows DLL resolution and signing remain unverified. See the [Windows distribution guide](docs/cli/windows-distribution.md) for exact dependencies and limits.
A follow-up environment check identified no accessible Windows machine/VM, and the local Windows SDK, MSVC CRT headers and MSVC linker were unavailable. No MSVC candidate was built, run or inspected for DLL imports in that check. The guide includes a procedure for an available Windows environment.

### Source installation

These commands describe this source checkout. Unpublished changes are not included in historical public tags; see the [release checklist](docs/ci/oss-cli-release.md).

Building from source requires Rust 1.92.0 or later. The repository also provides `nix develop`.
A C toolchain is required for native dependencies. A normal source installation targets its build environment; static Linux distribution uses the dedicated musl target and verification steps.

```bash
git clone https://github.com/mt4110/veil-rs.git
cd veil-rs
cargo install --locked --path crates/veil-cli
veil scan .
veil scan . --format json
veil scan . --fail-on-findings 1
```

JSON has `schemaVersion: "veil-v1"`; diagnostics go to stderr. Results are masked by default.
CLI `--unsafe` and `--mask-mode plain` reveal raw secrets: avoid these modes in stored or shared reports. Config-file `output.mask_mode = "plain"` is rejected with exit code 2.

**Published 0.21.0-rc.1 limitation:** The ZIP's table format prints raw matches even with `--mask-mode redact`. This source fixes the issue, but existing ZIPs are unchanged. Until a corrected binary is distributed, use text or JSON with `--mask-mode redact` for shared reports.
`--fail-on-findings N` fails when the new count is at least N: use `1` to fail on any finding. `0` is invalid input and returns exit code 2.
Exit codes: `0` means the configured finding thresholds were not exceeded, `1` means a threshold was exceeded, and `2` means an execution error or a file-count truncation.
In normal non-interactive filesystem scans, finding-count/file-count truncation, oversized text files and read/UTF-8 errors return 2. Baseline suppression retains finding-limit completeness even when no new findings remain. Check `limit_reached`, `file_limit_reached`, `max_file_size_reached` and `read_error_reached` in the summary. Ignored or binary files may be skipped, so exit code 0 does not prove a complete scan.

## Useful commands

```bash
veil --help
veil init --non-interactive
veil rules list
veil rules explain creds.aws.access_key_id
veil config check
veil doctor
```

Initialization is optional and adds configured failure thresholds. Review the generated configuration.
[Baselines](docs/baseline/usage.md) suppress matching findings. Creation rejects finding/file-count truncation with exit code 2 and preserves an existing baseline. Matching does not identify changes to raw secret values. Baselines are not risk approvals or proof of remediation.
`veil filter` masks stdin. Preview in-place masking with `veil mask --dry-run` first.

## Local rules and migration

Use `core.rules_dir` for a local RulePack. Old `core.remote_rules_url` / `VEIL_REMOTE_RULES_URL` settings produce an explicit migration error rather than silently reducing rule coverage.
Built-in rules require no update. To install a local signed pack:

```bash
veil update --local-pack rules.zip --public-key publisher.hex
```

The matching `rules.sig` and an explicitly trusted hex-encoded Ed25519 public key are required. Signatures and expiration are checked.
Existing `.veil/rules` is preserved. Configure another rules directory or apply the pack in a fresh project.

Trial installation budgets are 16 MiB per archive, 4 MiB per expanded entry, 64 MiB total expanded contents, and 4096 entries. Signed packs exceeding a budget are rejected with exit code 2.

Installation validates the signature, expiry, and rule contents before publishing the directory. Packs with an explicit expiry are also rejected by scans and rule listing after expiration. It does not automatically activate the pack or rewrite your configuration. Add the following value to the existing `[core]` section of the configuration you use, then confirm the added rules with `veil rules list` (or `veil --config <path> rules list` for another configuration file).

```toml
[core]
rules_dir = ".veil/rules"
```

## Build boundary

Default workspace commands target `veil-config`, `veil-core`, and `veil-cli`.
The existing `veil verify` command validates Evidence Pack v1 in both default and minimal builds, including hashes, optional external run_meta anchors, completeness and resource budgets. Its existing input contract is preserved.
New ledger/Detect/Prove/Decide, PatchGate/VEX and signed-v2 contracts are outside this source candidate. Legacy UI/server/Guardian sources are retained without becoming normal CLI dependencies.

See [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE).
Third-party licenses, copyright notices and included source archives are listed in [THIRD_PARTY_NOTICES](THIRD_PARTY_NOTICES.md) and [NOTICE](NOTICE).


## Preserved main features (2026-10-13)

Built-in presets, interactive scans and `rules promote-templates` remain available; inspect each command's help. `logs-jp` requires the local log pack generated by `veil init --preset logs-jp`. Partial masking hides secrets of eight characters or fewer entirely and shows only the last four characters of longer secrets.
`veil lsp --preset fintech-jp` starts a separate `veil-lsp` over stdio and forwards the effective configuration. Install it from source with `cargo install --locked --path crates/veil-lsp` and add it to PATH. The CLI ZIP does not include that server; normal scans do not start or link it.
