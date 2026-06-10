#!/usr/bin/env python3
"""Normalize legacy Optional spellings in compiler-diagnosed source files.

The Optional bridge has been deleted, so the v2 self-compiler is now the type
oracle: files that still use legacy `Some`/`None` for Optional produce
`VariantNotFound` diagnostics.  This codemod first asks `regen_stage0 --verify`
for that diagnostic set, then rewrites only identifier tokens in those files:

    Some -> Present
    None -> Absent

It is not a blind repository text replacement.  Files without compiler
diagnostics are untouched, and tokens inside comments or strings are ignored.
Lowercase `none` is a separate language literal and is not changed.
"""

from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
REGEN_CMD = ["cargo", "run", "-p", "v2-compiler", "--bin", "regen_stage0", "--", "--verify"]
DIAG_RE = re.compile(
    r"^variant '(?:Some|None)' not found in type '.+' "
    r"\((?P<path>[^():]+\.dag):\d+-\d+\)$"
)
IDENT_CHARS = set("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_")
REPLACEMENTS = {"Some": "Present", "None": "Absent"}


def run_regen_verify() -> str:
    env = os.environ.copy()
    env.setdefault("CARGO_TERM_COLOR", "never")
    proc = subprocess.run(
        REGEN_CMD,
        cwd=REPO_ROOT,
        env=env,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=False,
    )
    return proc.stdout


def diagnosed_paths(output: str) -> list[Path]:
    paths: set[Path] = set()
    for line in output.splitlines():
        match = DIAG_RE.match(line.strip())
        if not match:
            continue
        rel_path = Path(match.group("path"))
        if rel_path.is_absolute() or ".." in rel_path.parts:
            raise RuntimeError(f"refusing suspicious diagnostic path: {rel_path}")
        paths.add(REPO_ROOT / rel_path)
    return sorted(paths)


def is_ident_boundary(text: str, start: int, end: int) -> bool:
    before_ok = start == 0 or text[start - 1] not in IDENT_CHARS
    after_ok = end == len(text) or text[end] not in IDENT_CHARS
    return before_ok and after_ok


def rewrite_source(text: str) -> tuple[str, int]:
    out: list[str] = []
    i = 0
    edits = 0
    in_string = False
    escape = False
    in_line_comment = False

    while i < len(text):
        ch = text[i]

        if in_line_comment:
            out.append(ch)
            if ch == "\n":
                in_line_comment = False
            i += 1
            continue

        if in_string:
            out.append(ch)
            if escape:
                escape = False
            elif ch == "\\":
                escape = True
            elif ch == '"':
                in_string = False
            i += 1
            continue

        if text.startswith("//", i):
            out.append("//")
            i += 2
            in_line_comment = True
            continue

        if ch == '"':
            out.append(ch)
            i += 1
            in_string = True
            continue

        replaced = False
        for old, new in REPLACEMENTS.items():
            end = i + len(old)
            if text.startswith(old, i) and is_ident_boundary(text, i, end):
                out.append(new)
                edits += 1
                i = end
                replaced = True
                break
        if replaced:
            continue

        out.append(ch)
        i += 1

    return "".join(out), edits


def apply(paths: list[Path], dry_run: bool) -> int:
    total = 0
    for path in paths:
        text = path.read_text()
        rewritten, edits = rewrite_source(text)
        if edits and not dry_run:
            path.write_text(rewritten)
        if edits:
            print(f"{path.relative_to(REPO_ROOT)}: {edits}")
        total += edits
    return total


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Rewrite legacy Optional Some/None spellings in compiler-diagnosed files."
    )
    parser.add_argument("--dry-run", action="store_true", help="report edits without writing files")
    args = parser.parse_args()

    output = run_regen_verify()
    paths = diagnosed_paths(output)
    if not paths:
        print("normalize_optional_match_surface: no diagnosed Optional legacy files found")
        return 0

    total = apply(paths, dry_run=args.dry_run)
    mode = "would rewrite" if args.dry_run else "rewrote"
    print(
        f"normalize_optional_match_surface: {mode} {total} Some/None token(s) "
        f"in {len(paths)} compiler-diagnosed file(s)"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
