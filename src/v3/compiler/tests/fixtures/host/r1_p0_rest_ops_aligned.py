#!/usr/bin/env python3
"""R1C-B / T-P0 — host receipt for `p0_rest_ops_aligned`.

Mirrors `create_comment_rest_path_matches_github_issues_comments_api` in
`src/v2/tests/src/effects.rs`: github.Pulls CreateComment must use the Issues API path
(`/issues/{issue_number}/comments`), not pulls/{pull_number}/comments.
"""
from __future__ import annotations

import pathlib
import sys


def main() -> int:
    root = pathlib.Path(__file__).resolve().parents[6]
    path = root / "dsl" / "extdeps" / "github" / "pulls.dag"
    text = path.read_text(encoding="utf-8")
    needle = "operation CreateComment"
    if needle not in text:
        print("CreateComment operation missing", file=sys.stderr)
        return 1
    i = text.index(needle)
    chunk = text[i : i + 1500]
    if "/issues/" not in chunk:
        print("CreateComment chunk missing /issues/ path segment", file=sys.stderr)
        return 1
    if "issue_number" not in chunk:
        print("CreateComment chunk missing issue_number path param", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
