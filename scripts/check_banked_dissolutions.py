#!/usr/bin/env python3
"""Banked-dissolutions ratchet."""

from __future__ import annotations

import re
import sys

from ci_process import repo_root


def parse_forbidden(master_text: str) -> list[str]:
    match = re.search(r"^FORBIDDEN=\(\n(?P<body>.*?)^\)$", master_text, re.MULTILINE | re.DOTALL)
    if not match:
        return []
    return re.findall(r'"([^"]+)"', match.group("body"))


def main() -> int:
    root = repo_root()
    master_plan = root / "docs/post-l15-phase-plan.md"
    if not master_plan.exists():
        print(f"banked-dissolutions: missing master plan {master_plan.relative_to(root)}", file=sys.stderr)
        return 1

    forbidden = parse_forbidden(master_plan.read_text(encoding="utf-8"))
    if not forbidden:
        print(
            "banked-dissolutions: could not extract FORBIDDEN array from docs/post-l15-phase-plan.md",
            file=sys.stderr,
        )
        print(
            "The master plan's § Banked dissolutions block must contain a bash-style `FORBIDDEN=(...)` array.",
            file=sys.stderr,
        )
        return 1

    files = sorted(
        {
            *root.glob("docs/lane*.md"),
            *root.glob("docs/phase*.md"),
        }
    )
    files = [
        path
        for path in files
        if path.name != "post-l15-phase-plan.md" and not path.name.startswith("design-")
    ]
    if not files:
        print("banked-dissolutions: no lane/phase docs to scan")
        return 0

    violations = 0
    for pattern in forbidden:
        hits: list[str] = []
        for path in files:
            rel = path.relative_to(root).as_posix()
            for idx, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
                if pattern in line:
                    hits.append(f"{rel}:{idx}:{line}")
        if hits:
            if violations == 0:
                print("banked-dissolutions ratchet: forbidden shapes found in lane/phase docs.")
                print("Authority: docs/post-l15-phase-plan.md § Banked dissolutions.\n")
            print(f"--- forbidden: {pattern} ---")
            print("\n".join(hits))
            print()
            violations += 1

    if violations:
        print(f"banked-dissolutions ratchet: {violations} forbidden shape(s) found.")
        print("Fix: delete the restatement and reference the DB doc instead.")
        return 1

    print(
        f"banked-dissolutions ratchet: clean ({len(files)} docs scanned, "
        f"{len(forbidden)} forbidden shapes from docs/post-l15-phase-plan.md)"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
