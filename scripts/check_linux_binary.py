#!/usr/bin/env python3
"""Inspect ELF linkage without treating an absent loader as proof of runtime compatibility."""
from __future__ import annotations

import os
from pathlib import Path
import re
import subprocess


def parse_elf(header: str, programs: str, dynamic: str, versions: str, *, require_static: bool = False) -> dict:
    machine = re.search(r"^\s*Machine:\s*(.+)$", header, re.M)
    if not machine or "Program Headers:" not in programs:
        raise ValueError("missing ELF header/program information")
    interpreter = bool(re.search(r"^\s*INTERP\s", programs, re.M))
    needed = re.findall(r"\(NEEDED\)\s+Shared library: \[([^\]]+)\]", dynamic)
    glibc = sorted(set(re.findall(r"GLIBC_(\d+(?:\.\d+)+)", versions)), key=lambda v: tuple(map(int, v.split("."))))
    static = not interpreter and not needed and "GLIBC_" not in versions
    if require_static and not static:
        raise ValueError("static Linux candidate has an interpreter, shared dependencies or GLIBC references")
    return {"elf_machine": machine[1].strip(), "static_verified": static,
            "has_interpreter": interpreter, "needed_libraries": needed,
            "glibc_versions": glibc, "glibc_symbol_floor": glibc[-1] if glibc else None}


def inspect_linux_binary(binary: Path, *, require_static: bool = False) -> dict:
    with binary.open("rb") as stream:
        if stream.read(4) != b"\x7fELF":
            raise ValueError("Linux candidate is not an ELF executable")
    env = os.environ.copy()
    env["LC_ALL"] = "C"
    reports = [subprocess.run(["readelf", flag, str(binary)], capture_output=True, text=True,
                              check=True, timeout=30, env=env).stdout
               for flag in ("-hW", "-lW", "-dW", "--version-info")]
    return parse_elf(*reports, require_static=require_static)


def check_musl_link_map(text: str) -> dict:
    if re.search(r"\blibgcc(?:_eh)?\.a\(", text):
        raise ValueError("GNU libgcc archive contributed code; audit its runtime license before distribution")
    for archive in ("libc.a", "libunwind.a"):
        if not re.search(r"/rustlib/[^\s]+/lib/self-contained/" + re.escape(archive) + r"\(", text):
            raise ValueError(f"missing Rust self-contained {archive} contribution in link map")
    return {"libc": "Rust self-contained musl", "unwinder": "Rust self-contained LLVM libunwind",
            "gnu_libgcc_archive_contribution": False}


def execution_context(mode: str, runtime_arch: str, elf_machine: str | None,
                      host_arch: str | None = None, emulator: str | None = None) -> dict:
    aliases = {"aarch64": "aarch64", "arm64": "aarch64", "amd64": "x86_64", "x64": "x86_64",
               "x86_64": "x86_64", "advanced micro devices x86-64": "x86_64"}
    normalize = lambda value: aliases.get(value.casefold(), value.casefold()) if value else None
    host, runtime, binary = normalize(host_arch), normalize(runtime_arch), normalize(elf_machine)
    if mode not in {"native", "emulated", "unknown"}:
        raise ValueError("unsupported execution mode")
    if mode == "native" and (not host or emulator or host != runtime or (binary and binary != host)):
        raise ValueError("native execution requires matching declared host, runtime and ELF architecture, without emulator")
    if mode == "emulated" and (not host or not emulator):
        raise ValueError("emulated execution requires the actual host architecture and emulator declaration")
    return {"mode": mode, "host_architecture": host, "runtime_architecture": runtime,
            "binary_architecture": binary, "emulator": emulator, "mode_source": "caller declaration"}
