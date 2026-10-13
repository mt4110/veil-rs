#!/usr/bin/env python3
"""Validate the reviewed default CLI license inventory; never discover bundle files by glob."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path, PurePosixPath
import re
import subprocess
import tomllib
from urllib.parse import urlsplit

MANIFEST = "third_party/manifest.json"
SOURCE_PACKAGES = {"colored", "libgit2-sys"}


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def regular_file(root: Path, name: str) -> Path:
    path = root / name
    if any(p.is_symlink() for p in (path, *path.parents) if p != root):
        raise ValueError(f"symlink in third-party payload: {name}")
    if not path.is_file() or path.stat().st_size == 0:
        raise ValueError(f"missing third-party payload: {name}")
    return path


def package_ids(tree: str) -> set[tuple[str, str]]:
    result = set()
    for line in tree.splitlines():
        match = re.match(r"^([A-Za-z0-9_-]+) v([^ |]+)", line)
        if not match:
            raise ValueError(f"unrecognized Cargo dependency entry: {line}")
        if match[1] not in {"veil-cli", "veil-core", "veil-config"}:
            result.add((match[1], match[2]))
        elif " (" not in line:
            raise ValueError("unexpected non-workspace veil dependency")
    return result


def validate(root: Path, *, check_lock: bool = False) -> dict:
    root = root.resolve(strict=True)
    data = json.loads(regular_file(root, MANIFEST).read_text(encoding="utf-8"))
    if data["schemaVersion"] != "veil-third-party-v1" or data["profile"] != "veil-cli/default/normal,build/all-targets":
        raise ValueError("unsupported third-party inventory profile")
    packages = data["packages"]
    identities = [(p["name"], p["version"]) for p in packages]
    if len(set(identities)) != len(identities):
        raise ValueError("duplicate third-party package")
    owners = {f"{name}-{version}" for name, version in identities}
    native = data.get("native_components", [])
    native_ids = [p["id"] for p in native]
    if len(native_ids) != len(set(native_ids)) or any(not re.fullmatch(r"[a-z0-9][a-z0-9._-]+", n) for n in native_ids):
        raise ValueError("invalid native component identity")
    owners.update(native_ids)
    payloads = data["files"]
    names = set()
    for entry in payloads:
        name = entry["path"]
        parts = PurePosixPath(name).parts
        if ("\\" in name or "%" in name or PurePosixPath(name).is_absolute()
                or any(p.startswith(".") for p in parts)
                or PurePosixPath(name).as_posix() != name or name in names):
            raise ValueError(f"invalid third-party path: {name}")
        if len(parts) < 4 or parts[:2] != ("third_party", "licenses") or not name.endswith(".txt"):
            if len(parts) != 3 or parts[:2] != ("third_party", "sources"):
                raise ValueError(f"unapproved third-party payload: {name}")
            if not any(parts[2] == f"{p['name']}-{p['version']}.crate" and p["name"] in SOURCE_PACKAGES for p in packages):
                raise ValueError(f"unapproved source archive: {name}")
        elif parts[2] not in owners | {f"rust-{v}" for v in data["rust_versions"]}:
            raise ValueError(f"unapproved license owner: {name}")
        elif not (re.match(r"^(LICENSE|LICENCE|COPYING|NOTICE|COPYRIGHT|AUTHORS|SOURCE-ATTRIBUTION)(?:[._-]|$)", parts[-1], re.I)
                  or parts[-1] == "COPYING3.txt"
                  or (parts[-2] == "LICENSES" and parts[-1] in {"Apache-2.0.txt.txt", "MIT.txt.txt"})):
            raise ValueError(f"unapproved attribution filename: {name}")
        if not re.fullmatch(r"[0-9a-f]{64}", entry["sha256"]):
            raise ValueError("invalid payload checksum")
        origin = urlsplit(entry["origin"])
        if origin.scheme != "https" or not origin.netloc:
            raise ValueError("license origin must be a public HTTPS URL")
        if digest(regular_file(root, name)) != entry["sha256"]:
            raise ValueError(f"third-party checksum mismatch: {name}")
        names.add(name)
    referenced = set()
    for p in packages:
        if not p["license"] or not p["license_files"] or not re.fullmatch(r"[0-9a-f]{64}", p["checksum"]):
            raise ValueError(f"incomplete package license: {p['name']}")
        prefix = f"third_party/licenses/{p['name']}-{p['version']}/"
        if any(n not in names or not n.startswith(prefix) for n in p["license_files"]):
            raise ValueError("invalid package license reference")
        referenced.update(p["license_files"])
        source = p.get("source_archive")
        if p["name"] in SOURCE_PACKAGES and not source:
            raise ValueError(f"required source archive missing: {p['name']}")
        if source:
            if source not in names or source != f"third_party/sources/{p['name']}-{p['version']}.crate":
                raise ValueError("invalid source archive reference")
            if digest(regular_file(root, source)) != p["checksum"]:
                raise ValueError("source archive differs from Cargo.lock checksum")
            referenced.add(source)
    for component in native:
        prefix = f"third_party/licenses/{component['id']}/"
        if not component["license"] or not component["license_files"] or any(n not in names or not n.startswith(prefix) for n in component["license_files"]):
            raise ValueError("invalid native component license reference")
        referenced.update(component["license_files"])
    for version in data["rust_versions"]:
        required = f"third_party/licenses/rust-{version}/COPYRIGHT-library.html.txt"
        if required not in names:
            raise ValueError(f"missing Rust standard library notices: {version}")
        referenced.update(n for n in names if n.startswith(f"third_party/licenses/rust-{version}/"))
    if referenced != names:
        raise ValueError("unreferenced third-party payload")
    expected = names | {MANIFEST}
    directories = {p.as_posix() for n in expected for p in Path(n).parents if p != Path(".")}
    for path in (root / "third_party").rglob("*"):
        name = path.relative_to(root).as_posix()
        if path.is_symlink() or (name not in expected and not (path.is_dir() and name in directories)):
            raise ValueError(f"unapproved third-party entry: {name}")
    if check_lock:
        lock = regular_file(root, "Cargo.lock")
        if digest(lock) != data["cargo_lock_sha256"]:
            raise ValueError("Cargo.lock changed; refresh the license inventory")
        locked = {(p["name"], p["version"]): p for p in tomllib.loads(lock.read_text())["package"]}
        for p in packages:
            found = locked.get((p["name"], p["version"]), {})
            if found.get("checksum") != p["checksum"] or found.get("source") != "registry+https://github.com/rust-lang/crates.io-index":
                raise ValueError(f"license inventory differs from Cargo.lock: {p['name']}")
    return data


def check_dependencies(root: Path) -> None:
    data = validate(root, check_lock=True)
    tree = subprocess.run(["cargo", "tree", "--locked", "-p", "veil-cli", "--target", "all",
                           "-e", "normal,build", "--prefix", "none", "--format", "{p}"],
                          cwd=root, check=True, capture_output=True, text=True, timeout=120).stdout
    if package_ids(tree) != {(p["name"], p["version"]) for p in data["packages"]}:
        raise ValueError("default CLI dependency graph changed; refresh the license inventory")


def check_rust_version(root: Path, version: str) -> None:
    if version not in validate(root)["rust_versions"]:
        raise ValueError(f"Rust {version} has no reviewed standard library license packet; refresh before packaging")


if __name__ == "__main__":
    try:
        check_dependencies(Path(__file__).resolve().parents[1])
    except (OSError, ValueError, KeyError, subprocess.SubprocessError) as error:
        raise SystemExit(f"Third-party license check failed: {error}")
    print("Third-party license inventory and payloads verified")
