#!/usr/bin/env python3
"""Ensure every v3 integration test target is represented in CI timing capture."""

from __future__ import annotations

import json
import re
import subprocess
import sys

from ci_process import gha_error, repo_root


def main() -> int:
    root = repo_root()
    workflow = root / ".github/workflows/ci.yml"
    if not workflow.exists():
        gha_error(f"missing {workflow.relative_to(root)}")
        return 1

    proc = subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        cwd=root,
        check=True,
        stdout=subprocess.PIPE,
        text=True,
    )
    data = json.loads(proc.stdout)
    names: set[str] = set()
    for pkg in data.get("packages", []):
        if pkg.get("name") != "v3-compiler":
            continue
        for target in pkg.get("targets", []):
            if target.get("kind") == ["test"]:
                names.add(target["name"])
    if not names:
        gha_error("no v3-compiler integration test targets in cargo metadata")
        return 1

    workflow_text = workflow.read_text(encoding="utf-8")
    failed = False
    for name in sorted(names):
        pattern = rf"cargo test -p v3-compiler --test {re.escape(name)}[ \t].*--report-time"
        if not re.search(pattern, workflow_text):
            gha_error(
                f"v3-compiler integration test target '{name}' has no split full-suite step "
                f"with --report-time in {workflow.relative_to(root)}. Add a step "
                "(mirror determinism_test / integration) or fold the module into an existing tests/*.rs harness."
            )
            failed = True
    if failed:
        return 1

    print("v3 full-suite split covers all cargo integration test targets: OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
