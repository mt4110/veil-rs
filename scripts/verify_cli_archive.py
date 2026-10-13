#!/usr/bin/env python3
"""Verify and run an extracted local CLI ZIP with an explicit execution-mode record; upload nothing."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import stat
import subprocess
import tempfile
import zipfile

from check_cli_release import (ROOT, PACKAGES, bundle_files, check_binary, check_bundle,
                               check_metadata, regular_document, release_documents)
from check_third_party import check_dependencies, check_rust_version, validate
from check_linux_binary import inspect_linux_binary, check_musl_link_map, execution_context
from check_windows_binary import inspect_windows_binary, check_windows_gnu_link_map


def verify_archive(root: Path, archive: Path, binary_name: str, version: str, *, require_static: bool = False, windows_runner: str | None = None) -> dict:
    if windows_runner and (binary_name != "veil.exe" or platform.system() != "Linux"):
        raise ValueError("Wine verification requires a Windows binary and a Linux host")
    if require_static and binary_name == "veil.exe":
        raise ValueError("static Linux verification cannot check a Windows PE")
    if require_static and platform.system() != "Linux":
        raise ValueError("static Linux verification requires a Linux host")
    allowed = bundle_files(binary_name, root)
    if len({n.casefold() for n in allowed}) != len(allowed):
        raise ValueError("bundle paths collide on case-insensitive filesystems")
    if archive.is_symlink() or not archive.is_file():
        raise ValueError("archive must be a regular file")
    with zipfile.ZipFile(archive) as zipped, tempfile.TemporaryDirectory(prefix="veil-archive-check-") as scratch:
        entries = zipped.infolist()
        names = [entry.filename for entry in entries]
        if len(names) != len(set(names)) or set(names) != allowed:
            raise ValueError("ZIP file set does not match the allowlist")
        extracted = Path(scratch)
        for entry in entries:
            name = entry.filename
            mode = entry.external_attr >> 16
            if stat.S_IFMT(mode) not in (0, stat.S_IFREG) or entry.flag_bits & 1:
                raise ValueError(f"ZIP entry must be an unencrypted regular file: {name}")
            limit = 128 * 1024 * 1024 if name == binary_name else 256 * 1024 if name == "SHA256SUMS" else regular_document(root, name).stat().st_size
            if entry.file_size <= 0 or entry.file_size > limit:
                raise ValueError(f"invalid ZIP entry size: {name}")
            if entry.compress_type not in (zipfile.ZIP_STORED, zipfile.ZIP_DEFLATED):
                raise ValueError(f"unsupported ZIP compression: {name}")
            destination = extracted / name
            destination.parent.mkdir(parents=True, exist_ok=True)
            with zipped.open(entry) as source, destination.open("xb") as target:
                shutil.copyfileobj(source, target)
            if name == binary_name and os.name != "nt":
                if not mode & stat.S_IXUSR:
                    raise ValueError("ZIP CLI has lost its executable permission")
                destination.chmod(mode & 0o777)
        check_bundle(extracted, binary_name)
        for name in release_documents(root):
            if (extracted / name).read_bytes() != (root / name).read_bytes():
                raise ValueError(f"ZIP document differs from the current candidate: {name}")
        # Explicit Wine execution is a compatibility check, not Windows-native evidence.
        windows = inspect_windows_binary(extracted / binary_name) if binary_name == "veil.exe" else None
        if windows_runner:
            check_binary(extracted / binary_name, version, windows_runner=windows_runner)
        else:
            check_binary(extracted / binary_name, version)
        inventory = validate(extracted)
        linux = inspect_linux_binary(extracted / binary_name, require_static=require_static) if platform.system() == "Linux" and not windows else None
        if require_static and not {"musl-1.2.3", "llvm-libunwind-rust-1.92.0"} <= {p["id"] for p in inventory.get("native_components", [])}:
            raise ValueError("static Linux runtime license packet is missing")
        return {
            "schemaVersion": "veil-archive-check-v1", "status": "passed", "version": version,
            "platform": platform.system(), "architecture": platform.machine(),
            "archive_sha256": hashlib.sha256(archive.read_bytes()).hexdigest(),
            "binary_sha256": hashlib.sha256((extracted / binary_name).read_bytes()).hexdigest(),
            "archive_entries": len(entries), "checksum_entries": len(allowed) - 1,
            "third_party_packages": len(inventory["packages"]),
            "third_party_payloads": len(inventory["files"]),
            "native_components": len(inventory.get("native_components", [])), "linux_elf": linux,
            "windows_pe": windows,
            "checks": ["exact ZIP file set", "regular entries and executable permission", "SHA256SUMS",
                       "license originals and source checksums", "document and attribution links",
                       "documents match current source", "extracted CLI execution"],
        }


def source_fingerprint(root: Path) -> str:
    names = set(release_documents(root)) | {"Cargo.toml", "Cargo.lock", "rust-toolchain.toml", "flake.nix", "crates/veil-lsp/Cargo.toml"}
    for package in PACKAGES:
        names.add(f"crates/{package}/Cargo.toml")
        for directory in ("src", "tests"):
            names.update(p.relative_to(root).as_posix() for p in (root / "crates" / package / directory).rglob("*") if p.is_file())
        if (root / "crates" / package / "build.rs").exists():
            names.add(f"crates/{package}/build.rs")
    names.update(p.relative_to(root).as_posix() for p in (root / "crates/veil/rules").rglob("*") if p.is_file())
    names.update(p.relative_to(root).as_posix() for p in (root / "scripts").glob("*check*.py"))
    names.add("scripts/verify_cli_archive.py")
    names.update(p.relative_to(root).as_posix() for p in (root / ".github/workflows").glob("*.yml"))
    if (root / ".gitattributes").exists():
        names.add(".gitattributes")
    digest = hashlib.sha256()
    for name in sorted(names):
        digest.update(name.encode() + b"\0" + hashlib.sha256(regular_document(root, name).read_bytes()).digest())
    return digest.hexdigest()


def write_report(path: Path, report: dict) -> None:
    # An existing verification record remains intact.
    with path.open("x", encoding="utf-8", newline="\n") as output:
        json.dump(report, output, indent=2, ensure_ascii=False)
        output.write("\n")


def archive_execution_context(report: dict, mode: str, host: str | None, emulator: str | None, windows_runner: str | None) -> dict:
    if windows_runner:
        if report["platform"] != "Linux" or not report.get("windows_pe") or mode != "emulated":
            raise ValueError("Wine run requires Linux, PE and explicit emulated mode")
        result = execution_context(mode, report["architecture"], report["windows_pe"]["machine"], host, emulator)
        result["compatibility_layer"] = "Wine"
        result["windows_native"] = False
        return result
    if report.get("windows_pe") and report["platform"] != "Windows":
        raise ValueError("Windows-native verification requires a Windows host or explicit Wine runner")
    result = execution_context(mode, report["architecture"], (report.get("windows_pe") or {}).get("machine") or (report.get("linux_elf") or {}).get("elf_machine"), host, emulator)
    if report.get("windows_pe"):
        result["windows_native"] = mode == "native"
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--archive", type=Path, required=True)
    parser.add_argument("--binary-name", choices=("veil", "veil.exe"), default="veil.exe" if os.name == "nt" else "veil")
    parser.add_argument("--report", type=Path)
    parser.add_argument("--require-static", action="store_true", help="reject ELF interpreter, DT_NEEDED or GLIBC references on Linux")
    parser.add_argument("--link-map", type=Path, help="link map from the exact static candidate build")
    parser.add_argument("--execution-mode", choices=("native", "emulated", "unknown"), default="unknown")
    parser.add_argument("--execution-host-architecture", help="actual execution host ISA, checked outside any emulator")
    parser.add_argument("--emulator", help="emulator identity; required for emulated execution")
    parser.add_argument("--wine-runner", help="explicit Wine executable (Linux compatibility execution only, requires emulated mode)")
    parser.add_argument("--windows-gnu-link-map", type=Path, help="exact Windows GNU candidate link map; requires reviewed MinGW/GCC notices")
    args = parser.parse_args()
    if args.require_static != bool(args.link_map):
        parser.error("--require-static and --link-map must be supplied together")
    try:
        version = check_metadata(ROOT)
        check_dependencies(ROOT)
        rustc = subprocess.run(["rustc", "-vV"], capture_output=True, text=True, check=True, timeout=30).stdout
        check_rust_version(ROOT, rustc.splitlines()[0].split()[1])
        if args.require_static and rustc.splitlines()[0].split()[1] != "1.92.0":
            raise ValueError("musl runtime packet is reviewed for Rust 1.92.0; refresh before changing compiler")
        if args.wine_runner and args.execution_mode != "emulated":
            raise ValueError("Wine must not be recorded as native Windows execution")
        if args.windows_gnu_link_map:
            if args.binary_name != "veil.exe" or args.require_static or rustc.splitlines()[0].split()[1] != "1.92.0":
                raise ValueError("Windows GNU runtime audit requires Windows PE and Rust 1.92.0")
            inventory = validate(ROOT)
            if not {"mingw-w64-12.0.0", "gcc-runtime-14.2.0"} <= {p["id"] for p in inventory.get("native_components", [])}:
                raise ValueError("Windows GNU runtime license packet is missing")
            if args.windows_gnu_link_map.is_symlink() or not args.windows_gnu_link_map.is_file():
                raise ValueError("Windows GNU link map must be a regular file")
            windows_runtime = check_windows_gnu_link_map(args.windows_gnu_link_map.read_text(encoding="utf-8"))
        report = verify_archive(ROOT, args.archive, args.binary_name, version, require_static=args.require_static, windows_runner=args.wine_runner)
        report["execution"] = archive_execution_context(report, args.execution_mode, args.execution_host_architecture, args.emulator, args.wine_runner)
        if args.windows_gnu_link_map:
            report["windows_runtime"] = windows_runtime
            report["windows_link_map_sha256"] = hashlib.sha256(args.windows_gnu_link_map.read_bytes()).hexdigest()
        if args.link_map:
            if args.link_map.is_symlink() or not args.link_map.is_file():
                raise ValueError("link map must be a regular file")
            report["static_runtime"] = check_musl_link_map(args.link_map.read_text(encoding="utf-8"))
            report["link_map_sha256"] = hashlib.sha256(args.link_map.read_bytes()).hexdigest()
        report["rustc"] = rustc.strip()
        report["source_fingerprint"] = source_fingerprint(ROOT)
        revision = subprocess.run(["git", "rev-parse", "HEAD"], cwd=ROOT, capture_output=True, text=True, timeout=30)
        if revision.returncode == 0:
            dirty = subprocess.run(["git", "status", "--porcelain"], cwd=ROOT, capture_output=True, text=True, check=True, timeout=30).stdout
            report["source_revision"] = revision.stdout.strip()
            report["source_worktree_dirty"] = bool(dirty)
        else:
            report["source_revision"] = None
            report["source_worktree_dirty"] = None
            report["source_kind"] = "unversioned source copy"
        if args.report:
            write_report(args.report, report)
    except (OSError, ValueError, KeyError, zipfile.BadZipFile, subprocess.SubprocessError) as error:
        print(f"CLI archive verification failed: {error}")
        return 1
    print(f"CLI archive verified after extraction: {report['platform']} {report['architecture']}, {report['archive_entries']} files, mode={report['execution']['mode']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
