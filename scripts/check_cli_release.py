#!/usr/bin/env python3
"""Validate local CLI release metadata, smoke-test a binary, and stage an allowlisted bundle."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
import posixpath
import re
from pathlib import Path
import platform
import shutil
import subprocess
import tempfile
import tomllib
from html import unescape
from html.parser import HTMLParser
from urllib.parse import unquote, urlsplit
import zipfile
from check_third_party import MANIFEST, validate as validate_licenses, check_dependencies, check_rust_version
from check_windows_binary import inspect_windows_binary

ROOT = Path(__file__).resolve().parents[1]
PACKAGES = ("veil-config", "veil-core", "veil-cli")
ALIGNED_PACKAGES = (*PACKAGES, "veil-lsp")
# Reviewed explicitly; links never add new files to the bundle.
DOCUMENTS = (
    "README.md", "README_EN.md", "LICENSE-MIT", "LICENSE-APACHE",
    "NOTICE", "THIRD_PARTY_NOTICES.md", MANIFEST,
    "docs/README.md", "docs/SSOT.md", "docs/DEVELOPMENT.md", "docs/DEVELOPMENT_EN.md",
    "docs/TESTING_SECRETS.md", "docs/architecture/local-oss.md", "docs/ci/oss-cli-release.md",
    "docs/cli/README.md", "docs/cli/getting-started.md", "docs/cli/usage-guidelines.md",
    "docs/cli/linux-distribution.md",
    "docs/cli/windows-distribution.md",
    "docs/cli/scan.md", "docs/cli/init.md", "docs/baseline/usage.md", "docs/json-schema.md",
    "docs/security/threat_model.md",
)


def release_documents(root: Path) -> tuple[str, ...]:
    inventory = validate_licenses(root)
    return (*DOCUMENTS, *(entry["path"] for entry in inventory["files"]))


def regular_document(root: Path, name: str) -> Path:
    root = root.resolve(strict=True)
    path = root / name
    for parent in (path, *path.parents):
        if parent == root:
            break
        if parent.is_symlink():
            raise ValueError(f"symlink in release document: {name}")
    if not path.is_file() or path.stat().st_size == 0:
        raise ValueError(f"missing release document: {name}")
    if path.resolve(strict=True) != path:
        raise ValueError(f"redirected release document: {name}")
    return path


class HTMLLinks(HTMLParser):
    def __init__(self):
        super().__init__()
        self.links = []

    def handle_starttag(self, tag, attrs):
        for key, value in attrs:
            if key in ("href", "src") and value is not None:
                self.links.append(value)


def document_links(text: str) -> list[str]:
    # Fenced/inline examples are not navigational links.
    text = re.sub(r"(?ms)^ *(`{3,}|~{3,})[^\n]*\n.*?^ *\1 *$", "", text)
    text = re.sub(r"`[^`\n]*`", "", text)
    text = re.sub(r"<!--.*?-->", "", text, flags=re.S)
    references = dict((key.strip().casefold(), value) for key, value in re.findall(
        r'^ *\[([^\]]+)\]: *<?([^ >\n]+)>?', text, re.M
    ))
    inline_links = re.compile(r'!?\[[^\]\n]*\]\( *<?([^ >)\n]+)>?(?: +[\"\'][^\n]*?[\"\'])? *\)')
    links = inline_links.findall(text)
    if re.search(r'\]\(', inline_links.sub("", text)):
        raise ValueError("unsupported or malformed inline Markdown link")
    for label, key in re.findall(r'!?\[([^\]\n]+)\]\[([^\]\n]*)\]', text):
        key = (key or label).strip().casefold()
        if key not in references:
            raise ValueError(f"undefined Markdown link reference: {key}")
        links.append(references[key])
    # All declared references are checked, including shortcut references.
    links.extend(references.values())
    parser = HTMLLinks()
    parser.feed(text)
    links.extend(parser.links)
    return links


def check_document_links(root: Path) -> None:
    documents = release_documents(root)
    allowed = set(documents)
    notice_targets = set()
    for name in documents:
        path = regular_document(root, name)
        if not name.endswith(".md"):
            continue
        for link in document_links(path.read_text(encoding="utf-8")):
            url = urlsplit(unescape(link))
            if url.scheme in ("http", "https", "mailto"):
                continue  # Availability of external services is not checked offline.
            if url.scheme or url.netloc or url.query:
                raise ValueError(f"unsupported bundle link in {name}: {link}")
            target = unquote(url.path)
            if "\\" in target or target.startswith("/"):
                raise ValueError(f"absolute bundle link in {name}: {link}")
            relative = posixpath.normpath(posixpath.join(posixpath.dirname(name), target)) if target else name
            if relative not in allowed:
                raise ValueError(f"bundle link leaves the document allowlist: {name} -> {link}")
            regular_document(root, relative)
            if name == "THIRD_PARTY_NOTICES.md":
                notice_targets.add(relative)
            if url.fragment:
                content = regular_document(root, relative).read_text(encoding="utf-8")
                content = re.sub(r"(?ms)^ *(`{3,}|~{3,})[^\n]*\n.*?^ *\1 *$", "", content)
                anchors = set(re.findall(r'(?:id|name)=[\"\']([^\"\']+)[\"\']', content))
                seen = {}
                for heading in re.findall(r'^ {0,3}#{1,6} +(.+?) *#* *$', content, re.M):
                    heading = re.sub(r'<[^>]+>', '', heading).strip().lower()
                    slug = ''.join(c for c in heading if c.isalnum() or c in '_- ')
                    slug = slug.replace(' ', '-')
                    index = seen.get(slug, 0)
                    seen[slug] = index + 1
                    anchors.add(f"{slug}-{index}" if index else slug)
                if unquote(url.fragment) not in anchors:
                    raise ValueError(f"missing bundle anchor: {name} -> {link}")
    required = set(documents) - set(DOCUMENTS) | {"NOTICE", MANIFEST}
    if not required <= notice_targets:
        raise ValueError("third-party index must link every license, source archive, NOTICE and manifest")



def check_macos_linkage(output: str) -> None:
    lines = output.splitlines()
    if len(lines) < 2 or not lines[0].endswith(":"):
        raise ValueError("could not read standalone macOS dependency information")
    for line in lines[1:]:
        dependency = line.strip().split(" (compatibility version", 1)[0]
        if dependency and not dependency.startswith(("/usr/lib/", "/System/Library/")):
            raise ValueError(f"standalone macOS CLI has a non-system dependency: {dependency}")


def check_metadata(root: Path) -> str:
    workspace = tomllib.loads((root / "Cargo.toml").read_text(encoding="utf-8"))["workspace"]["package"]
    version = workspace["version"]
    for name in ALIGNED_PACKAGES:
        package = tomllib.loads((root / "crates" / name / "Cargo.toml").read_text(encoding="utf-8"))["package"]
        if package["version"] != version:
            raise ValueError(f"{name}: version differs from workspace {version}")
        if package.get("rust-version") != {"workspace": True}:
            raise ValueError(f"{name}: must inherit workspace rust-version")
    nix_text = (root / "flake.nix").read_text(encoding="utf-8")
    nix_versions = re.findall(r'pname\s*=\s*"veil";\s*version\s*=\s*"([^"\n]+)";', nix_text)
    if nix_versions != [version]:
        raise ValueError("Nix CLI version differs from workspace")
    toolchain = tomllib.loads((root / "rust-toolchain.toml").read_text(encoding="utf-8"))["toolchain"]["channel"]
    if toolchain != workspace["rust-version"]:
        raise ValueError("rust-toolchain and workspace rust-version differ")
    check_document_links(root)
    return version


def check_binary(binary: Path, version: str, *, windows_runner: str | None = None) -> None:
    binary = binary.resolve(strict=True)
    if binary.name == "veil.exe":
        inspect_windows_binary(binary)
    elif platform.system() == "Darwin":
        linkage = subprocess.run(["otool", "-L", str(binary)], capture_output=True, text=True, timeout=30, check=True)
        check_macos_linkage(linkage.stdout)
    with tempfile.TemporaryDirectory(prefix="veil-release-check-") as scratch:
        root = Path(scratch)
        env = os.environ.copy()
        for name in ("VEIL_REMOTE_RULES_URL", "VEIL_ORG_CONFIG", "VEIL_USER_CONFIG", "VEIL_ORG_RULES", "VEIL_FAIL_SCORE"):
            env.pop(name, None)
        env["XDG_CONFIG_HOME"] = str(root / "settings")

        def run(*args: str) -> subprocess.CompletedProcess:
            prefix = [windows_runner] if windows_runner else []
            return subprocess.run([*prefix, str(binary), *args], cwd=root, env=env, capture_output=True, text=True, encoding="utf-8", timeout=30)

        result = run("--version")
        if result.returncode != 0 or result.stdout.strip() != f"veil {version}":
            raise ValueError("release binary version mismatch")
        help_result = run("--help")
        commands = {line.split()[0] for line in help_result.stdout.splitlines() if line.startswith("  ") and line.split()}
        if help_result.returncode != 0 or not {"scan", "verify"} <= commands or commands & {"detect", "prove", "decide", "case", "guardian", "legacy"}:
            raise ValueError("release binary must use the default local OSS profile")
        verification = run("verify", "missing.zip", "--format", "json")
        if verification.returncode != 2 or json.loads(verification.stdout).get("status") != "INVALID":
            raise ValueError("release binary must preserve existing v1 verification")
        token = "AKIA" + "A" * 16
        (root / "sample.txt").write_text(f"private-neighbor-marker\naws_key={token}\n")
        result = run("scan", "sample.txt", "--format", "json", "--mask-mode", "redact")
        if result.returncode != 0:
            raise ValueError("release scan failed")
        report = json.loads(result.stdout)
        if report["schemaVersion"] != "veil-v1" or report["summary"]["scanned_files"] != 1 or not report["findings"]:
            raise ValueError("release JSON scan contract failed")
        if token in result.stdout or "private-neighbor-marker" in result.stdout:
            raise ValueError("release JSON exposes raw data")
        if (root / ".veil").exists():
            raise ValueError("normal scan unexpectedly created governance state")
        result = run("scan", "sample.txt", "--format", "json", "--mask-mode", "redact", "--fail-on-findings", "1")
        if result.returncode != 1 or token in result.stdout:
            raise ValueError("release binary must fail the finding threshold with masked output")
        # Exercise actual release executables on each CI OS, including table's
        # default feature and its text fallback in minimal builds.
        (root / "safe.toml").write_text("", encoding="utf-8")
        (root / "plain.toml").write_text('[output]\nmask_mode = "plain"\n', encoding="utf-8")
        for format_name in ("text", "json", "html", "md", "markdown", "table"):
            for config_name, options, reveal_raw in (
                ("safe.toml", (), False),
                ("safe.toml", ("--mask-mode", "redact"), False),
                ("safe.toml", ("--mask-mode", "partial"), False),
                ("safe.toml", ("--unsafe",), True),
                ("safe.toml", ("--mask-mode", "plain"), True),
                ("plain.toml", (), False),
                ("plain.toml", ("--mask-mode", "redact"), False),
            ):
                result = run("--config", config_name, "scan", "sample.txt", "--format", format_name,
                             "--no-color", "--fail-on-findings", "1", *options)
                expected_exit = 2 if config_name == "plain.toml" else 1
                valid_report = ("creds.aws.access_key_id" in result.stdout) if expected_exit == 1 else not result.stdout
                if (result.returncode != expected_exit or not valid_report
                        or (token in result.stdout) != reveal_raw or token in result.stderr):
                    raise ValueError(f"release output masking contract failed: {format_name}, {config_name}, {options}")
        result = run("scan", "missing", "--format", "json")
        if result.returncode != 2 or result.stdout:
            raise ValueError("release binary must reject missing paths")


def bundle_files(binary_name: str, root: Path) -> set[str]:
    if binary_name not in ("veil", "veil.exe"):
        raise ValueError("unsupported bundle binary name")
    return set(release_documents(root)) | {binary_name, "SHA256SUMS"}


def check_bundle(root: Path, binary_name: str) -> None:
    allowed = bundle_files(binary_name, root)
    actual = set()
    directories = {parent.as_posix() for name in allowed for parent in Path(name).parents if parent != Path(".")}
    for path in root.rglob("*"):
        name = path.relative_to(root).as_posix()
        if path.is_symlink():
            raise ValueError(f"symlink in bundle: {name}")
        if path.is_dir() and name in directories:
            continue
        if not path.is_file() or name not in allowed:
            raise ValueError(f"unapproved bundle entry: {name}")
        actual.add(name)
    if actual != allowed:
        raise ValueError("bundle file set does not match the allowlist")
    check_document_links(root)
    sums = {}
    for line in (root / "SHA256SUMS").read_text(encoding="utf-8").splitlines():
        digest, name = line.split("  ", 1)
        if name not in allowed - {"SHA256SUMS"} or name in sums:
            raise ValueError("invalid bundle checksum entry")
        sums[name] = digest
    if set(sums) != allowed - {"SHA256SUMS"}:
        raise ValueError("incomplete bundle checksums")
    for name, expected in sums.items():
        with (root / name).open("rb") as file:
            if hashlib.file_digest(file, "sha256").hexdigest() != expected:
                raise ValueError(f"bundle checksum mismatch: {name}")


def build_bundle(root: Path, binary: Path, output: Path) -> None:
    check_document_links(root)
    sources = {name: regular_document(root, name) for name in release_documents(root)}
    binary_name = "veil.exe" if binary.suffix == ".exe" else "veil"
    sources[binary_name] = binary
    if binary.is_symlink() or not binary.is_file():
        raise ValueError("bundle binary must be a regular file")
    output.mkdir(parents=True, exist_ok=False)
    sums = []
    for name, source in sorted(sources.items()):
        destination = output / name
        destination.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, destination)
        with destination.open("rb") as file:
            digest = hashlib.file_digest(file, "sha256").hexdigest()
        sums.append(f"{digest}  {name}\n")
    (output / "SHA256SUMS").write_text("".join(sums), encoding="utf-8")
    check_bundle(output, binary_name)


def build_archive(bundle: Path, binary_name: str, output: Path) -> None:
    check_bundle(bundle, binary_name)
    if output.resolve().is_relative_to(bundle.resolve()):
        raise ValueError("archive output must be outside the staged bundle")
    # Exclusive creation protects an existing archive. Only named files are written.
    with zipfile.ZipFile(output, "x", compression=zipfile.ZIP_DEFLATED) as archive:
        for name in sorted(bundle_files(binary_name, bundle)):
            archive.write(bundle / name, name)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path)
    parser.add_argument("--bundle", type=Path)
    parser.add_argument("--archive", type=Path, help="create a checked ZIP from --bundle")
    args = parser.parse_args()
    if args.bundle and not args.binary:
        parser.error("--bundle requires --binary")
    if args.archive and not args.bundle:
        parser.error("--archive requires --bundle and --binary")
    try:
        version = check_metadata(ROOT)
        check_dependencies(ROOT)
        if args.binary:
            check_binary(args.binary, version)
        if args.bundle:
            rustc = subprocess.run(["rustc", "--version"], capture_output=True, text=True, check=True, timeout=30)
            check_rust_version(ROOT, rustc.stdout.split()[1])
            build_bundle(ROOT, args.binary, args.bundle)
        if args.archive:
            build_archive(args.bundle, "veil.exe" if args.binary.suffix == ".exe" else "veil", args.archive)
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        print(f"CLI release check failed: {error}")
        return 1
    print(f"CLI release checks passed: {version}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
