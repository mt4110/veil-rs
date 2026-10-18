# First scan, staged check and next use

Status: `reference`

[日本語](getting-started.md) · [Installation, PATH and rollback](install.en.md)

[0.22.1](https://github.com/mt4110/veil-rs/releases/tag/v0.22.1) is published as a stable release.
It includes two narrow false-positive fixes, the offline installer and bilingual help.
Download the native ZIP and SHA256SUMS from that release, or follow the
[Ubuntu download and checksum instructions](getting-started.md#ubuntuで自分のコードを試す).
Bundled documents are fixed build-time snapshots and may still describe preparation;
use this release's assets and the current guide. Published ZIPs remain unchanged.

## Install 0.22.1 and open a new terminal

Supported archives: Linux x86_64 musl and macOS ARM64. Before extracting into a new
directory, compare the exact ZIP with its outer SHA256SUMS and check its source.
Checksums alone do not authenticate a publisher; use a trusted key to verify a
signed tag and its recorded ZIP hashes. Keep the bundled license texts and notices.

From the extracted archive directory:

```sh
sh install.sh --lang en
```

Follow the printed PATH line in the active shell startup file as explained in the
[installation guide](install.en.md#set-path-once). The installer does not edit
dotfiles, use sudo or download anything. Rust and Python are not needed to install
or scan. Git is needed only for staged checks. For a conflicting existing CLI,
use a separate prefix; do not overwrite it. The guide also explains explicit
activation and returning to a retained version.

Open a new terminal and check resolution, version and help:

```sh
command -v veil
veil --version
veil --lang en scan --help
veil_bin=$(command -v veil)
```

Confirm the expected prefix and release version `0.22.1`. Stop if resolution or
startup fails. `--lang ja|en` selects help, not all reports or runtime messages.
Priority is explicit option, VEIL_LANG, LC_ALL, LC_MESSAGES, LANG, then English.
See the [language details](install.en.md#cli-help-language).
macOS is an ad-hoc-signed trial asset: Developer ID signing, notarization and
downloaded Gatekeeper behavior remain unverified. Do not disable OS protection.

## Scan one of your own files

Replace the project path and `main.py` with an existing file you own:

```sh
cd "/absolute/path/to/your-project" &&
"$veil_bin" scan "main.py" --mask-mode redact --no-color --fail-on-findings 1
scan_status=$?
printf 'Exit code: %s\n' "$scan_status"
```

If `cd` failed, no scan ran. Otherwise interpret the result:

| Exit | Meaning | Next action |
| --- | --- | --- |
| 0 | The selected failure thresholds were not reached | Check scanned and skipped inputs before expanding the scope |
| 1 | A completed scan reached the findings threshold | Classify locally; revoke/rotate real credentials and fix the source as needed |
| 2 | Error or incomplete inspection | Resolve the input, configuration or limit problem, then rerun |

Success means completing a scan, understanding its result and choosing the next
action. Zero findings is not proof of safety. No initialization, baseline or
RulePack download is required. `scan` does not rewrite source files. Organization,
user and repository settings, ignores and VEIL_* environment variables affect
results; preserve them and use `veil config check` to diagnose problems.

This scans current files, not all Git history. Hidden/ignored files can be outside
directory scans; `scan .` does not prove that `.env` was inspected. Binary files
are skipped. Redaction can leave paths and undetected strings: do not publish
source code, credentials or complete reports to complete this procedure.

## Check the index before committing

Inside the Git project, select only files intended for the commit:

```sh
git status --short
git add -- main.py
git diff --cached --name-only
```

Stop on any Git error and verify the list. Then run:

```sh
"$veil_bin" scan --staged --mask-mode redact --no-color --fail-on-findings 1
scan_status=$?
printf 'Exit code: %s\n' "$scan_status"
```

`--staged` reads the entire index content of added/modified files relative to HEAD,
using Git's selected index (including GIT_INDEX_FILE). It does not inspect only
added lines or automatically stage working files. All selected index entries are
in scope; do not assume a positional path narrows a staged scan.

- Exit 0 can also mean an empty index change. Check the file list and inspected count.
- Exit 1 requires local classification. After fixing a file, **restage it with
  `git add -- main.py`**, then scan again. Fixing only the working copy leaves the
  old index content in place.
- Exit 2 means error/incomplete; do not treat it as a clean scan.
- Unstaged/untracked/unchanged files, ignored files, binary files and submodule
  contents are not covered by this staged procedure. Force-adding an ignored file
  does not guarantee inspection. An explicit normal file scan can inspect an
  ignored text file when needed; do not erase ignores or baselines just to pass.

The scanner preserves the index and source. This procedure does not commit or
push. A clean staged check is not a guarantee that a commit is safe. Automated
hooks are optional; understand the manual result before installing one.

## Use the same CLI on the next real change

In the next terminal, check `command -v veil` and `veil --version` again, set
`veil_bin=$(command -v veil)`, and enter the project. Repeat file selection,
`git add`, list review, staged scan, repair and restaging. A new file can also be
scanned directly with the first command above. Do not invent a code change just
to claim repeat use: synthetic acceptance and real development reuse are separate.
