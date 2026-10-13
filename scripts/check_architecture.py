#!/usr/bin/env python3
"""Check the supported local OSS profile, including transitive build dependencies."""
from __future__ import annotations

import json
from pathlib import Path
import re
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
SUPPORTED = {"veil-config", "veil-core", "veil-cli"}
ALLOWED_LOCAL = {
    "veil-config": set(),
    "veil-core": {"veil-config"},
    "veil-cli": {"veil-core", "veil-config"},
}
FORBIDDEN = {"reqwest", "ureq", "hyper", "tokio", "axum", "oauth2", "tauri", "openssl-sys", "libssh2-sys"}


def forbidden(name: str) -> bool:
    return name in FORBIDDEN or name.startswith(("sqlx", "tauri-", "veil-telemetry"))


def check_metadata(metadata: dict) -> list[str]:
    packages = {p["id"]: p for p in metadata["packages"]}
    defaults = {packages[p]["name"] for p in metadata["workspace_default_members"]}
    errors = []
    if defaults != SUPPORTED:
        errors.append(f"default workspace must be {sorted(SUPPORTED)}, found {sorted(defaults)}")
    for package in packages.values():
        name = package["name"]
        if name not in SUPPORTED:
            continue
        for dep in package["dependencies"]:
            if dep["kind"] == "dev":
                continue
            if forbidden(dep["name"]):
                errors.append(f"{name}: forbidden dependency {dep['name']}")
            if dep["name"] == "git2" and (
                dep.get("uses_default_features", True)
                or set(dep.get("features", [])) & {"https", "ssh", "vendored-openssl"}
            ):
                errors.append(f"{name}: Git network transports must be disabled")
            if dep.get("path") and dep["name"] not in ALLOWED_LOCAL[name]:
                errors.append(f"{name}: forbidden local dependency {dep['name']}")
    return errors


def check_tree(package: str, output: str) -> list[str]:
    errors = []
    for line in output.splitlines():
        name = line.split()[0] if line.strip() else ""
        if name == "libgit2-sys":
            native = re.search(r"\+(\d+)\.(\d+)\.(\d+)", line)
            if native is None or tuple(map(int, native.groups())) < (1, 9, 5):
                errors.append(f"{package}: libgit2 1.9.5 or newer is required for the local pack-reading fix")
        if forbidden(name) or (name.startswith("veil-") and name not in SUPPORTED):
            errors.append(f"{package}: forbidden transitive dependency {name}")
    return sorted(set(errors))


def run_cargo(*args: str) -> str:
    return subprocess.check_output(
        ["cargo", *args, "--locked", "--offline"], cwd=ROOT, text=True
    )



EXCLUDED_RELEASE_PATHS = (
    ".private", ".design", "crates/veil-core/src/breaker.rs", "crates/veil-core/src/case",
    "crates/veil-cli/src/commands/detect.rs", "crates/veil-cli/src/commands/prove.rs",
    "crates/veil-cli/src/commands/decide.rs", "crates/veil-cli/src/commands/case.rs",
    "schemas/attestation_v2.schema.json", "schemas/case_ledger_v1.schema.json",
    "schemas/patch_result_v1.schema.json", "schemas/remediation_task_v1.schema.json",
    "schemas/vex_export_v1.schema.json",
)


def check_public_scope(root: Path) -> list[str]:
    errors = [f"excluded new release source: {name}" for name in EXCLUDED_RELEASE_PATHS if (root / name).exists()]
    symbols = {
        "crates/veil-core/src/lib.rs": r"\bCircuitBreaker\b|pub\s+mod\s+case\b",
        "crates/veil-cli/src/cli.rs": r"\bLegacyCommand\b|\b(?:DetectArgs|ProveArgs|DecideArgs|CaseCommand)\b",
        "crates/veil-cli/src/main.rs": r"Commands::(?:Detect|Prove|Decide|Case|Legacy)\b",
    }
    for name, pattern in symbols.items():
        path = root / name
        if path.is_file() and re.search(pattern, path.read_text()):
            errors.append(f"excluded new release API: {name}")
    return errors

def main() -> int:
    errors = check_public_scope(ROOT)
    try:
        metadata = json.loads(run_cargo("metadata", "--format-version", "1", "--no-deps"))
        errors.extend(check_metadata(metadata))
        for package in sorted(SUPPORTED):
            tree = run_cargo("tree", "-p", package, "--edges", "normal,build", "--prefix", "none")
            errors.extend(check_tree(package, tree))
    except (subprocess.CalledProcessError, KeyError, json.JSONDecodeError) as error:
        print(f"architecture check could not complete: {error}", file=sys.stderr)
        return 1

    network = re.compile(r"\b(?:reqwest|ureq|oauth2|tauri|sqlx)\s*::|\bstd\s*::\s*net\b|\b(?:TcpStream|UdpSocket|UnixStream)\b")
    for package in sorted(SUPPORTED):
        for path in sorted((ROOT / "crates" / package / "src").rglob("*.rs")):
            # Retained source for the retired CLI adapter; not declared by commands/mod.rs.
            if path.relative_to(ROOT).as_posix() == "crates/veil-cli/src/commands/guardian.rs":
                continue
            if network.search(path.read_text()):
                errors.append(f"{path.relative_to(ROOT)}: network/service code in local profile")
    if errors:
        for error in sorted(set(errors)):
            print(f"FAIL: {error}", file=sys.stderr)
        return 1
    print("architecture: local OSS crate boundaries and dependency graphs passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
