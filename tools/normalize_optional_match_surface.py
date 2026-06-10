#!/usr/bin/env python3
"""Normalize legacy Optional spellings diagnosed by the v2 self-compiler.

The Optional surface is canonicalized to `Present` / `Absent`.  This codemod is
deliberately compiler-driven: it first runs `regen_stage0 --verify`, collects
hard diagnostics that can only arise after the legacy Optional constructors are
removed from the bootstrap seed, and rewrites identifier tokens in those
diagnosed source files:

    Some -> Present
    None -> Absent

String literals, comments, and lowercase `none` are untouched.  The diagnostic
list is printed in dry-run and normal modes so the corpus sweep is auditable.
"""

from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[1]
REGEN_CMD = ["cargo", "run", "-p", "v2-compiler", "--bin", "regen_stage0", "--", "--verify"]
DIAG_RE = re.compile(
    r"^(?P<kind>variant|type) '(?P<name>Some|None)' not found in "
    r"(?:(?:type|scope) '(?P<type>[^']*)'|scope) "
    r"\((?P<path>[^():]+\.dag):(?P<start>\d+)-(?P<end>\d+)\)$"
)
IDENT_CHARS = set("abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_")
REPLACEMENTS = {"Some": "Present", "None": "Absent"}


@dataclass(frozen=True, order=True)
class Diagnostic:
    path: Path
    kind: str
    name: str
    type_name: str
    start: int
    end: int


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


def parse_diagnostics(output: str) -> list[Diagnostic]:
    diagnostics: set[Diagnostic] = set()
    for line in output.splitlines():
        match = DIAG_RE.match(line.strip())
        if not match:
            continue
        rel_path = Path(match.group("path"))
        if rel_path.is_absolute() or ".." in rel_path.parts:
            raise RuntimeError(f"refusing suspicious diagnostic path: {rel_path}")
        diagnostics.add(Diagnostic(
            path=REPO_ROOT / rel_path,
            kind=match.group("kind"),
            name=match.group("name"),
            type_name=match.group("type") or "",
            start=int(match.group("start")),
            end=int(match.group("end")),
        ))
    return sorted(diagnostics)


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

        for old, new in REPLACEMENTS.items():
            end = i + len(old)
            if text.startswith(old, i) and is_ident_boundary(text, i, end):
                out.append(new)
                edits += 1
                i = end
                break
        else:
            out.append(ch)
            i += 1

    return "".join(out), edits


def apply(diagnostics: list[Diagnostic], dry_run: bool) -> int:
    by_path: dict[Path, list[Diagnostic]] = {}
    for diagnostic in diagnostics:
        by_path.setdefault(diagnostic.path, []).append(diagnostic)

    total = 0
    for path in sorted(by_path):
        rel = path.relative_to(REPO_ROOT)
        reasons = ", ".join(
            f"{d.name}/{d.type_name or d.kind}@{d.start}-{d.end}" for d in by_path[path][:5]
        )
        if len(by_path[path]) > 5:
            reasons = f"{reasons}, +{len(by_path[path]) - 5} more"
        text = path.read_text()
        rewritten, edits = rewrite_source(text)
        if edits and not dry_run:
            path.write_text(rewritten)
        if edits:
            print(f"{rel}: {edits} ({reasons})")
        total += edits
    return total


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Rewrite legacy Optional Some/None spellings in compiler-diagnosed files."
    )
    parser.add_argument("--dry-run", action="store_true", help="report edits without writing files")
    args = parser.parse_args()

    diagnostics = parse_diagnostics(run_regen_verify())
    if not diagnostics:
        print("normalize_optional_match_surface: no legacy Optional diagnostics found")
        return 0

    total = apply(diagnostics, dry_run=args.dry_run)
    mode = "would rewrite" if args.dry_run else "rewrote"
    print(
        f"normalize_optional_match_surface: {mode} {total} Some/None token(s) "
        f"in {len({d.path for d in diagnostics})} compiler-diagnosed file(s)"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
