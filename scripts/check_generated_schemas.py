#!/usr/bin/env python3
import difflib
import json
import pathlib
import subprocess
import sys
import tempfile
from typing import Any

try:
    import yaml
except ModuleNotFoundError as exc:
    raise SystemExit(
        "PyYAML is required for schema checks. Install it with: python -m pip install PyYAML"
    ) from exc


ROOT = pathlib.Path(__file__).resolve().parents[1]
SCHEMA_FILES = [
    "openapi.local-api.yaml",
    "json-schema.safe-finding-api.json",
    "json-schema.report.json",
    "json-schema.run-meta.json",
    "json-schema.finding.json",
]


def main() -> int:
    with tempfile.TemporaryDirectory() as tmp:
        tmp_path = pathlib.Path(tmp)
        subprocess.run(
            [
                "cargo",
                "run",
                "-p",
                "veil-pro",
                "--bin",
                "export_local_api_schema",
                "--",
                "--out-dir",
                str(tmp_path),
            ],
            cwd=ROOT,
            check=True,
        )

        failures: list[str] = []
        for name in SCHEMA_FILES:
            expected = ROOT / "schemas" / name
            actual = tmp_path / name
            failures.extend(compare_files(expected, actual))

        for name in SCHEMA_FILES:
            path = tmp_path / name
            if name.endswith(".json"):
                doc = json.loads(path.read_text())
                failures.extend(check_refs(doc, f"{name}#"))
            else:
                doc = yaml.safe_load(path.read_text())
                failures.extend(check_refs(doc, f"{name}#"))

        run_meta = json.loads((tmp_path / "json-schema.run-meta.json").read_text())
        failures.extend(check_run_meta_contract(run_meta))

        openapi = yaml.safe_load((tmp_path / "openapi.local-api.yaml").read_text())
        failures.extend(check_openapi_contract(openapi))

    if failures:
        for failure in failures:
            print(failure, file=sys.stderr)
        return 1
    print("OK: generated schemas match tracked files and internal refs resolve")
    return 0


def compare_files(expected: pathlib.Path, actual: pathlib.Path) -> list[str]:
    if not expected.exists():
        return [f"missing tracked schema: {expected}"]
    if not actual.exists():
        return [f"missing generated schema: {actual}"]

    expected_text = expected.read_text()
    actual_text = actual.read_text()
    if expected_text == actual_text:
        return []

    diff = "".join(
        difflib.unified_diff(
            expected_text.splitlines(keepends=True),
            actual_text.splitlines(keepends=True),
            fromfile=str(expected),
            tofile=str(actual),
        )
    )
    return [f"schema drift detected for {expected.name}:\n{diff}"]


def check_refs(doc: Any, label: str) -> list[str]:
    failures: list[str] = []

    def walk(value: Any, path: str) -> None:
        if isinstance(value, dict):
            ref = value.get("$ref")
            if isinstance(ref, str) and ref.startswith("#/"):
                if not resolve_pointer(doc, ref[1:]):
                    failures.append(f"unresolved ref in {label}{path}: {ref}")
            for key, child in value.items():
                walk(child, f"{path}/{escape_pointer(str(key))}")
        elif isinstance(value, list):
            for index, child in enumerate(value):
                walk(child, f"{path}/{index}")

    walk(doc, "")
    return failures


def resolve_pointer(doc: Any, pointer: str) -> bool:
    current = doc
    for raw_part in pointer.strip("/").split("/"):
        part = raw_part.replace("~1", "/").replace("~0", "~")
        if isinstance(current, dict) and part in current:
            current = current[part]
        elif isinstance(current, list) and part.isdigit() and int(part) < len(current):
            current = current[int(part)]
        else:
            return False
    return True


def escape_pointer(part: str) -> str:
    return part.replace("~", "~0").replace("/", "~1")


def check_run_meta_contract(schema: dict[str, Any]) -> list[str]:
    failures: list[str] = []
    result = find_schema_object(schema, "RunResultMeta")
    if result is None:
        result = schema.get("properties", {}).get("result")
    if not isinstance(result, dict):
        return ["RunMeta result schema not found"]

    required = result.get("required", [])
    if "limitReasons" not in required:
        failures.append("RunMeta.result.required must include limitReasons")
    if result.get("additionalProperties") is not False:
        failures.append("RunMeta.result.additionalProperties must be false")
    return failures


def find_schema_object(schema: dict[str, Any], name: str) -> dict[str, Any] | None:
    for defs_key in ("$defs", "definitions"):
        defs = schema.get(defs_key)
        if isinstance(defs, dict) and isinstance(defs.get(name), dict):
            return defs[name]
    return None


def check_openapi_contract(openapi: dict[str, Any]) -> list[str]:
    failures: list[str] = []
    if openapi.get("openapi") != "3.1.0":
        failures.append("OpenAPI version must be 3.1.0")
    required_paths = {
        "/api/me",
        "/api/projects",
        "/api/scan",
        "/api/runs/{runId}",
        "/api/runs/{runId}/evidence.zip",
        "/api/policy",
        "/api/baseline",
        "/api/doctor",
    }
    paths = set((openapi.get("paths") or {}).keys())
    missing = sorted(required_paths - paths)
    if missing:
        failures.append(f"OpenAPI missing paths: {', '.join(missing)}")

    result = (
        openapi.get("components", {})
        .get("schemas", {})
        .get("RunResultMeta", {})
    )
    if "limitReasons" not in result.get("required", []):
        failures.append("OpenAPI RunResultMeta.required must include limitReasons")
    if result.get("additionalProperties") is not False:
        failures.append("OpenAPI RunResultMeta.additionalProperties must be false")
    return failures


if __name__ == "__main__":
    raise SystemExit(main())
