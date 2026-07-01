#!/usr/bin/env python3
import pathlib
import subprocess
import sys


ROOT = pathlib.Path(__file__).resolve().parents[1]

COMMANDS = [
    ["cargo", "test", "--workspace"],
    ["npm", "--prefix", "crates/veil-pro/frontend", "run", "build"],
    [sys.executable, "scripts/check_generated_schemas.py"],
    [
        "cargo",
        "run",
        "-p",
        "veil-cli",
        "--",
        "verify",
        "tests/fixtures/evidence/golden.zip",
        "--require-complete",
    ],
]


def main() -> int:
    for command in COMMANDS:
        print("+ " + " ".join(command), flush=True)
        subprocess.run(command, cwd=ROOT, check=True)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
