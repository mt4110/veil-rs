#!/usr/bin/env python3
"""Read PE imports without a host toolchain. This is not a clean-Windows test.

Layout: https://learn.microsoft.com/en-us/windows/win32/debug/pe-format
"""
from pathlib import Path
import re
import struct

SYSTEM_DLLS = frozenset({
    "advapi32.dll", "bcrypt.dll", "bcryptprimitives.dll", "crypt32.dll", "gdi32.dll", "iphlpapi.dll",
    "kernel32.dll", "msvcrt.dll", "ntdll.dll", "ole32.dll", "oleaut32.dll",
    "psapi.dll", "rpcrt4.dll", "secur32.dll", "shell32.dll", "shlwapi.dll",
    "ucrtbase.dll", "user32.dll", "userenv.dll", "version.dll", "ws2_32.dll",
})


def inspect_windows_binary(path: Path) -> dict:
    data = path.read_bytes()

    def read(offset, fmt):
        size = struct.calcsize(fmt)
        if offset < 0 or offset + size > len(data):
            raise ValueError("truncated PE structure")
        return struct.unpack_from(fmt, data, offset)

    if data[:2] != b"MZ":
        raise ValueError("not a Windows PE executable")
    pe, = read(0x3c, "<I")
    if data[pe:pe + 4] != b"PE\0\0":
        raise ValueError("invalid PE signature")
    machine, sections = read(pe + 4, "<HH")
    optional_size, characteristics = read(pe + 20, "<HH")
    optional = pe + 24
    magic, = read(optional, "<H")
    if magic != 0x20b or machine not in (0x8664, 0xaa64) or not characteristics & 2 or characteristics & 0x2000:
        raise ValueError("expected a 64-bit Windows executable")
    if optional_size < 112 or optional + optional_size > len(data) or not 1 <= sections <= 96:
        raise ValueError("invalid PE header size or section count")
    image_base, = read(optional + 24, "<Q")
    directory_count, = read(optional + 108, "<I")
    if directory_count > 16 or 112 + 8 * directory_count > optional_size:
        raise ValueError("invalid PE data directories")
    ranges = []
    for index in range(sections):
        section = optional + optional_size + index * 40
        virtual_size, virtual, raw_size, raw = read(section + 8, "<IIII")
        read(section, "<40s")
        if raw_size and (raw + raw_size > len(data) or raw < optional + optional_size + sections * 40):
            raise ValueError("invalid PE section bounds")
        ranges.append((virtual, raw_size, raw))

    def offset(rva, length=1):
        matches = [raw + rva - virtual for virtual, size, raw in ranges if virtual <= rva and rva + length <= virtual + size]
        if len(matches) != 1:
            raise ValueError("PE RVA is unmapped or ambiguous")
        return matches[0]

    def dll_name(rva):
        result = bytearray()
        for index in range(256):
            byte = data[offset(rva + index)]
            if not byte:
                break
            result.append(byte)
        else:
            raise ValueError("unterminated PE DLL name")
        try:
            name = result.decode("ascii").lower()
        except UnicodeError as error:
            raise ValueError("invalid PE DLL name") from error
        if not re.fullmatch(r"[a-z0-9_.-]+\.dll", name):
            raise ValueError("invalid or redirected PE DLL name")
        return name

    def imports(index, width, delayed=False):
        if index >= directory_count:
            return []
        rva, size = read(optional + 112 + index * 8, "<II")
        if not rva and not size:
            return []
        if not rva or size < width or size > 1024 * 1024:
            raise ValueError("invalid PE import directory")
        offset(rva, size)
        names = []
        for cursor in range(0, size - width + 1, width):
            fields = read(offset(rva + cursor, width), "<" + "I" * (width // 4))
            if not any(fields):
                return sorted(set(names))
            if delayed:
                if fields[0] not in (0, 1):
                    raise ValueError("invalid delay-import attributes")
                name_rva = fields[1] if fields[0] == 1 else fields[1] - image_base
            else:
                name_rva = fields[3]
            names.append(dll_name(name_rva))
        raise ValueError("unterminated PE import directory")

    direct = imports(1, 20)
    delayed = imports(13, 32, True)
    if not direct:
        raise ValueError("Windows candidate has no normal imports")
    dependencies = sorted(set(direct + delayed))
    external = [name for name in dependencies if name not in SYSTEM_DLLS and not re.fullmatch(r"api-ms-win-(core|crt)-[a-z0-9-]+\.dll", name)]
    if external:
        raise ValueError("Windows candidate requires unbundled non-system DLLs: " + ", ".join(external))
    subsystem, = read(optional + 68, "<H")
    if subsystem != 3:
        raise ValueError("expected a Windows console executable")
    return {"format": "PE32+", "machine": "x86_64" if machine == 0x8664 else "aarch64",
            "subsystem": subsystem, "imports": direct, "delay_imports": delayed,
            "additional_dlls": external, "scope": "declared PE imports only; dynamic LoadLibrary and clean Windows remain separate checks"}


def check_windows_gnu_link_map(text: str) -> dict:
    archives = sorted(set(re.findall(r"^(/[^\n]+?\.a)\(", text, re.M)))
    for name in ("libmingw32.a", "libmingwex.a", "libgcc.a", "libgcc_eh.a"):
        if not any(path.endswith("/" + name) for path in archives):
            raise ValueError(f"missing Windows GNU runtime contribution: {name}")
    gcc = [path for path in archives if Path(path).name in {"libgcc.a", "libgcc_eh.a"}]
    if any("/x86_64-w64-mingw32/14-posix/" not in path for path in gcc):
        raise ValueError("Windows GNU runtime packet is reviewed for GCC 14 POSIX")
    return {"crt": "MinGW-w64 12.0.0 (build environment declaration)", "gcc": "14.2.0 (build environment declaration)",
            "contributing_archives": archives, "scope": "exact link map plus recorded build-tool versions; not inferred from DLL absence"}
