#!/usr/bin/env python3
"""Gate #103 workflow path-regex inventory ratchet."""

from __future__ import annotations

import re
import sys
from pathlib import Path

from ci_process import git_files, repo_root


def forbidden_patterns(path: Path) -> list[str]:
    patterns: list[str] = []
    for raw in path.read_text(encoding="utf-8").splitlines():
        item = raw.strip()
        if not item or item.startswith("#"):
            continue
        patterns.append(item)
    return patterns


def trigger_path_lines(path: Path) -> list[tuple[int, str]]:
    lines = path.read_text(encoding="utf-8").splitlines()
    matches: list[tuple[int, str]] = []
    in_on = False
    in_trigger = False
    trigger_indent = 0
    for idx, raw in enumerate(lines, start=1):
        if not raw.strip() or raw.lstrip().startswith("#"):
            continue
        indent = len(raw) - len(raw.lstrip(" "))
        content = raw.strip()
        if indent == 0:
            in_on = content == "on:"
            in_trigger = False
            continue
        if not in_on:
            continue
        if not in_trigger:
            if re.match(r"^(push|pull_request|pull_request_target):", content):
                in_trigger = True
                trigger_indent = indent
            continue
        if indent <= trigger_indent:
            if re.match(r"^(push|pull_request|pull_request_target):", content):
                in_trigger = True
                trigger_indent = indent
            else:
                in_trigger = False
            continue
        if re.match(r"^paths(-ignore)?:", content):
            matches.append((idx, raw))
    return matches


def main() -> int:
    root = repo_root()
    workflow_files = git_files([".github/workflows/*.yml", ".github/workflows/*.yaml"], root)
    forbidden = forbidden_patterns(root / "scripts/workflow-path-regex-forbidden-substrings.txt")
    violations = 0

    for path in workflow_files:
        rel = path.relative_to(root).as_posix()
        text = path.read_text(encoding="utf-8")
        for pattern in forbidden:
            for idx, line in enumerate(text.splitlines(), start=1):
                if pattern in line:
                    print(
                        "check-workflow-path-regex-inventory: "
                        f"FAIL: forbidden gate #103 workflow fingerprint in {rel} "
                        f"(matches `{pattern}`):\n{rel}:{idx}:{line}",
                        file=sys.stderr,
                    )
                    violations += 1
        for idx, line in trigger_path_lines(path):
            print(
                "check-workflow-path-regex-inventory: "
                f"FAIL: trigger-level paths:/paths-ignore: selection candidate: {rel}:{idx}:{line}",
                file=sys.stderr,
            )
            violations += 1
        for idx, line in enumerate(text.splitlines(), start=1):
            if re.search(r"dorny/paths-filter|tj-actions/changed-files|paths-filter@", line):
                print(
                    "check-workflow-path-regex-inventory: "
                    f"FAIL: changed-files action wrapper candidate: {rel}:{idx}:{line}",
                    file=sys.stderr,
                )
                violations += 1

    if violations:
        print(
            f"check-workflow-path-regex-inventory: {violations} violation(s) - see messages above.",
            file=sys.stderr,
        )
        return 1
    print("check-workflow-path-regex-inventory: ok (no authoritative path-regex selection in workflows)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
