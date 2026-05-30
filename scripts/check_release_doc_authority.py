#!/usr/bin/env python3
"""Release-doc authority consumer."""

from __future__ import annotations

import re
import sys
from pathlib import Path

from ci_process import repo_root


RELEASE_DOCS = [
    "docs/r2-structure.md",
    "docs/r3-structure.md",
    "docs/thesis/r2-r3-thesis-mapping.md",
]

FORBIDDEN_STRINGS = [
    "T-Ground-Engine",
    "T-Ground-Annotation",
    "canonical choice",
    "@target",
    "DECISIONS LOCKED",
    "T-Verification-L4L7",
]

RETRACTION_PATTERNS = [
    r"~~",
    r"🔄",
    r"SUPERSEDED",
    r"[Ss]upersedes",
    r"RETRACTED",
    r"[Rr]etracted",
    r"CLOSED 2026",
    r"[Rr]eplaces the retracted",
    r"the retracted",
    r"supersession",
    r"supersedes the prior",
    r"framing was retracted",
    r"\[retraction-context",
]


def line_is_retraction_context(line: str) -> bool:
    return any(re.search(pattern, line, flags=re.IGNORECASE) for pattern in RETRACTION_PATTERNS)


def check(root: Path) -> int:
    missing = [doc for doc in RELEASE_DOCS if not (root / doc).is_file()]
    if missing:
        print(f"Release-doc authority check FAILED: {len(missing)} configured release-control doc(s) missing.")
        print()
        for doc in missing:
            print(f"  MISSING: {doc}")
        print()
        print("Each missing doc was declared in RELEASE_DOCS in this script. The")
        print("consumer fails closed because silently skipping a missing doc would")
        print("let release-control authority shrink without review. Either:")
        print("  - the doc was renamed -> update RELEASE_DOCS to the new path")
        print("  - the doc was retired -> remove it from RELEASE_DOCS with")
        print("    the same attention as adding a new release-control authority")
        return 1

    violations = 0
    for doc in RELEASE_DOCS:
        path = root / doc
        for lineno, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
            for forbidden in FORBIDDEN_STRINGS:
                if forbidden not in line or line_is_retraction_context(line):
                    continue
                print(f"VIOLATION: {doc}:{lineno}")
                print(f"  forbidden string: '{forbidden}'")
                print(f"  line: {line}")
                print()
                violations += 1

    if violations:
        print(f"Release-doc authority check FAILED: {violations} violation(s) found.")
        print()
        print("Each violation is a forbidden lane/concept name appearing in a live")
        print("(non-retraction) context. Either:")
        print("  - the lane/concept is genuinely live, in which case remove it from")
        print("    FORBIDDEN_STRINGS in this script and update authority docs accordingly")
        print("  - the lane/concept is retracted, in which case add a retraction marker")
        print("    (~~strikethrough~~, SUPERSEDED, RETRACTED, 'replaces the retracted',")
        print("    or similar) to the line, OR remove the line entirely.")
        print()
        print('Authority: docs/r2-structure.md §"Release-doc authority discipline"')
        return 1

    print("Release-doc authority check passed: no forbidden stale concept names")
    print("in live sections of release-control docs.")
    return 0


def main() -> int:
    return check(repo_root())


if __name__ == "__main__":
    raise SystemExit(main())
