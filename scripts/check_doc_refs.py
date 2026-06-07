#!/usr/bin/env python3
"""Doc-reference resolver — the construction-tier enforcement for the doc authority rule.

Every Markdown link to a repo-relative path must resolve to a file that exists.
This is the standing detection that keeps `docs/thesis/doc-authority.md` (and the
whole doc tree) from silently re-accreting dangling references — the failure mode
that left doc-authority.md itself citing a ROADMAP structure that no longer exists.

Usage:
  check_doc_refs.py --all                 # scan every tracked .md (census/report)
  check_doc_refs.py FILE [FILE ...]       # scan specific files (CI: changed .md only)
  check_doc_refs.py --changed BASE        # scan .md changed vs BASE (e.g. origin/main)

Exit 0 if every checked file's links resolve; exit 1 if any dangling reference is found.
Touch-driven by design: CI runs it on the PR's changed .md files, so you fix a doc's
links when you touch it — no repo-wide sweep required, no ratchet baseline.

Out of scope (deliberately, for now): inline `backtick path` mentions and the
"every claimed canonical-home exists" check. Markdown links are the reference form
the rule governs; the rest is a future enhancement, not a silent gap.
"""
import os
import re
import subprocess
import sys

# [text](target) and ![alt](target). Capture the target up to the first ) or space.
LINK_RE = re.compile(r"!?\[[^\]]*\]\(\s*([^)\s]+)")

SKIP_PREFIXES = ("http://", "https://", "mailto:", "#", "tel:")


def repo_root() -> str:
    out = subprocess.run(
        ["git", "rev-parse", "--show-toplevel"],
        capture_output=True, text=True, check=True,
    )
    return out.stdout.strip()


def tracked_md(root: str) -> list[str]:
    out = subprocess.run(
        ["git", "ls-files", "*.md"],
        capture_output=True, text=True, check=True, cwd=root,
    )
    files = [f for f in out.stdout.splitlines() if f]
    return [f for f in files if not f.startswith(("target/", "node_modules/"))]


def changed_md(root: str, base: str) -> list[str]:
    out = subprocess.run(
        ["git", "diff", "--name-only", "--diff-filter=d", f"{base}...HEAD"],
        capture_output=True, text=True, check=True, cwd=root,
    )
    return [f for f in out.stdout.splitlines() if f.endswith(".md")]


def dangling_in_file(root: str, relpath: str) -> list[tuple[int, str]]:
    """Return (line_no, target) for each unresolved repo-relative link."""
    path = os.path.join(root, relpath)
    base_dir = os.path.dirname(path)
    findings: list[tuple[int, str]] = []
    with open(path, encoding="utf-8") as fh:
        for line_no, line in enumerate(fh, 1):
            for raw in LINK_RE.findall(line):
                target = raw.split("#", 1)[0]  # drop anchor
                if not target or target.startswith(SKIP_PREFIXES):
                    continue
                resolved = os.path.normpath(os.path.join(base_dir, target))
                if not os.path.exists(resolved):
                    findings.append((line_no, target))
    return findings


def main(argv: list[str]) -> int:
    root = repo_root()
    if argv and argv[0] == "--all":
        files = tracked_md(root)
    elif argv and argv[0] == "--changed":
        if len(argv) < 2:
            print("usage: check_doc_refs.py --changed BASE", file=sys.stderr)
            return 2
        files = changed_md(root, argv[1])
    elif argv:
        files = argv
    else:
        print(__doc__)
        return 2

    total = 0
    for relpath in files:
        for line_no, target in dangling_in_file(root, relpath):
            print(f"{relpath}:{line_no}: dangling reference -> {target}")
            total += 1

    n = len(files)
    if total:
        print(f"\nFAIL: {total} dangling reference(s) across {n} file(s).", file=sys.stderr)
        return 1
    print(f"OK: all references resolve across {n} file(s).")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
