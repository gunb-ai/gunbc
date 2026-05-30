#!/usr/bin/env python3
"""Fail if fabrication sentinel tokens reappear in compiler-facing sources."""

from __future__ import annotations

import sys

from ci_process import git_files, repo_root


SENTINEL = "__BUG_NO_PROFILE_"


def main() -> int:
    root = repo_root()
    violations = 0
    for path in git_files(["*.rs", "*.dag"], root):
        rel = path.relative_to(root).as_posix()
        if rel.startswith("docs/"):
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except UnicodeDecodeError:
            text = path.read_text(encoding="utf-8", errors="ignore")
        if SENTINEL in text:
            print(f"error: {SENTINEL} found in {rel}", file=sys.stderr)
            violations += 1

    if violations:
        print(f"check-fabrication-sentinels: failed ({violations} file(s))", file=sys.stderr)
        return 1
    print("check-fabrication-sentinels: ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
