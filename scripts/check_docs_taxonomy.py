#!/usr/bin/env python3
from __future__ import annotations

import argparse
import fnmatch
from collections import Counter
from pathlib import Path
from typing import Iterable


RULES = [
    ("docs/README.md", "active"),
    ("docs/SSOT.md", "active"),
    ("docs/DEVELOPMENT.md", "active"),
    ("docs/DEVELOPMENT_EN.md", "active"),
    ("docs/TESTING_SECRETS.md", "active"),
    ("docs/baseline/**", "reference"),
    ("docs/ci/**", "reference"),
    ("docs/cli/**", "reference"),
    ("docs/guardian/**", "reference"),
    ("docs/guardrails/**", "reference"),
    ("docs/integrations/**", "reference"),
    ("docs/json-schema.md", "reference"),
    ("docs/prompts/**", "reference"),
    ("docs/rules.md", "reference"),
    ("docs/rules/**", "reference"),
    ("docs/runbook/**", "reference"),
    ("docs/sales/**", "reference"),
    ("docs/security/threat_model.md", "reference"),
    ("docs/templates/**", "reference"),
    ("docs/pr/README.md", "reference"),
    ("docs/pr/sot_template.md", "reference"),
    ("docs/ops/*CONTRACT*.md", "reference"),
    ("docs/ops/LOCAL_STORAGE_POLICY_v1.md", "reference"),
    ("docs/ops/REVIEW_BUNDLE.md", "reference"),
    ("docs/dogfood/README.md", "reference"),
    ("docs/pr/_archive/**", "archive"),
    ("docs/pr/evidence/**", "archive"),
    ("docs/pr/PR-*.md", "archive"),
    ("docs/pr/PR-TBD-*.md", "archive"),
    ("docs/pr/*/implementation_plan.md", "archive"),
    ("docs/pr/*/plan.md", "archive"),
    ("docs/pr/*/task.md", "archive"),
    ("docs/ai/**", "archive"),
    ("docs/design/**", "archive"),
    ("docs/dogfood/**", "archive"),
    ("docs/evidence/**", "archive"),
    ("docs/epics/**", "archive"),
    ("docs/ops/**", "archive"),
    ("docs/perf/**", "archive"),
    ("docs/phase16/**", "archive"),
    ("docs/phase17/**", "archive"),
    ("docs/roadmap/**", "archive"),
    ("docs/security/dependabot/**", "archive"),
    ("docs/security_review.md", "archive"),
    ("docs/v0.20.0-planning/**", "archive"),
]


def iter_docs(root: Path) -> Iterable[Path]:
    yield from sorted((root / "docs").rglob("*.md"))


def classify(path: Path, root: Path) -> str | None:
    rel = path.relative_to(root).as_posix()
    for pattern, status in RULES:
        if fnmatch.fnmatch(rel, pattern):
            return status
    return None


def main() -> int:
    parser = argparse.ArgumentParser(description="Check docs active/reference/archive taxonomy.")
    parser.add_argument("--list", action="store_true", help="print every markdown file with status")
    args = parser.parse_args()

    root = Path(__file__).resolve().parents[1]
    counts: Counter[str] = Counter()
    unclassified: list[str] = []
    classified: list[tuple[str, str]] = []

    for path in iter_docs(root):
        status = classify(path, root)
        rel = path.relative_to(root).as_posix()
        if status is None:
            unclassified.append(rel)
            continue
        counts[status] += 1
        classified.append((rel, status))

    total = sum(counts.values()) + len(unclassified)
    print(f"docs taxonomy: total={total} active={counts['active']} reference={counts['reference']} archive={counts['archive']}")

    if args.list:
        for rel, status in classified:
            print(f"{status:9s} {rel}")

    if unclassified:
        print("unclassified:")
        for rel in unclassified:
            print(f"  {rel}")
        return 1

    print("all markdown docs are classified")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
