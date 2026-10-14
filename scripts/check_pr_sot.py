#!/usr/bin/env python3
"""Apply the same PR SOT policy to CI commits or the proposed Git index."""
from __future__ import annotations

import argparse
import re
import subprocess
import sys

SOT_PATH = re.compile(r"docs/pr/(?:PR-TBD-.*|PR-[0-9].*)\.md\Z")


def git(*args: str) -> bytes:
    return subprocess.check_output(["git", *args], stderr=subprocess.PIPE)


def check(base: str, head: str | None = None) -> int:
    revision = head or "HEAD"
    ancestor = git("merge-base", base, revision).decode().strip()
    args = ["diff", "--name-only", "-z", "--diff-filter=ACMRTD"]
    if head is None:
        args += ["--cached", ancestor]
    else:
        args += [ancestor, head]
    paths = [p.decode("utf-8", errors="surrogateescape") for p in git(*args).split(b"\0") if p]
    if not paths or all(p.startswith(("docs/", ".github/")) for p in paths):
        print("SOT guard: docs/CI only or no changes; SOT not required.")
        return 0
    for path in paths:
        if SOT_PATH.fullmatch(path):
            spec = f"{head}:{path}" if head else f":{path}"
            # Read only from the proposed snapshot, never from the working tree.
            try:
                git("cat-file", "-e", spec)
            except subprocess.CalledProcessError:
                continue
            print(f"SOT guard: changed SOT found: {path}")
            return 0
    print("SOT guard: missing changed docs/pr/PR-TBD-*.md or PR-<number>-*.md.", file=sys.stderr)
    print("README and changes outside docs/ or .github/ require a SOT in the PR diff.", file=sys.stderr)
    print("Describe SOT / What / Verification / Evidence, then git add the document.", file=sys.stderr)
    print(f"Compared against merge-base of {base} and {revision}; unstaged files do not count.", file=sys.stderr)
    return 1


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--base", help="PR base SHA/ref; local default: sot.baseRef or origin/main")
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--head", help="CI PR head SHA/ref")
    mode.add_argument("--staged", action="store_true", help="Check branch commits plus the proposed index")
    args = parser.parse_args()
    try:
        base = args.base
        if not base:
            config = subprocess.run(["git", "config", "--get", "sot.baseRef"], capture_output=True)
            if config.returncode not in (0, 1):
                raise RuntimeError("Cannot read sot.baseRef")
            base = config.stdout.decode().strip() or "origin/main"
        return check(base, args.head)
    except (subprocess.CalledProcessError, RuntimeError) as error:
        print(f"SOT guard: cannot inspect Git snapshots: {error}", file=sys.stderr)
        print("Fetch the PR base or set git config sot.baseRef <base-ref>; commit blocked.", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
