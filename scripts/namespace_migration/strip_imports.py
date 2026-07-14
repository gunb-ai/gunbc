#!/usr/bin/env python3
"""Remove import declaration blocks from .dag files (phase-3 corpus strip).

DOES NOT qualify bare references — run only after v1 global-unique fallback lands
and definer census rows are repointed. Default is dry-run.

Usage:
  python3 scripts/namespace_migration/strip_imports.py --dry-run
  python3 scripts/namespace_migration/strip_imports.py --apply --roots dag src
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

IMPORT_LINE = re.compile(r"^\s*import\s+")


def strip_imports_from_text(text: str) -> tuple[str, int]:
    lines = text.splitlines(keepends=True)
    out: list[str] = []
    removed = 0
    i = 0
    while i < len(lines):
        line = lines[i]
        if not IMPORT_LINE.match(line):
            out.append(line)
            i += 1
            continue
        removed += 1
        if "{" in line:
            while i < len(lines) and "}" not in lines[i]:
                i += 1
                removed += 1
            if i < len(lines):
                removed += 1
                i += 1
        else:
            i += 1
    # drop leading blank lines after module header
    while len(out) > 1 and out[1].strip() == "" and out[0].startswith("module "):
        out.pop(1)
    return "".join(out), removed


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--roots",
        nargs="+",
        default=["dag", "src"],
        help="Directories to process",
    )
    parser.add_argument(
        "--apply",
        action="store_true",
        help="Write changes (default: dry-run only)",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        default=True,
        help="Print stats without writing (default)",
    )
    args = parser.parse_args()
    if args.apply:
        args.dry_run = False

    repo = Path(__file__).resolve().parents[2]
    total_files = 0
    total_lines = 0
    for root_name in args.roots:
        root = repo / root_name
        if not root.is_dir():
            print(f"skip missing root: {root}", file=sys.stderr)
            continue
        for path in sorted(root.rglob("*.dag")):
            text = path.read_text(encoding="utf-8", errors="replace")
            if not any(IMPORT_LINE.match(line) for line in text.splitlines()):
                continue
            new_text, n = strip_imports_from_text(text)
            if n == 0:
                continue
            total_files += 1
            total_lines += n
            if args.dry_run:
                print(f"would strip {n} import lines from {path.relative_to(repo)}")
            else:
                path.write_text(new_text, encoding="utf-8")
                print(f"stripped {n} import lines from {path.relative_to(repo)}")

    mode = "dry-run" if args.dry_run else "applied"
    print(f"strip_imports [{mode}]: {total_lines} lines across {total_files} files")
    return 0


if __name__ == "__main__":
    sys.exit(main())
