#!/usr/bin/env python3
"""Normalize legacy Optional match arms reported by the v2 self-compiler.

This is intentionally diagnostic-scoped, not a text replacement.  With the
Optional spelling bridge deleted, the v2 compiler reports legacy `Some`/`None`
arms at exact source spans when an Optional match is authored with the old
surface.  This codemod rewrites only those reported variant tokens:

    Some -> Present
    None -> Absent

It leaves unrelated `Some`/`None` text, constructors, comments, examples, and
valid non-Optional coproducts untouched.
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
    r"^variant '(?P<variant>Some|None)' not found in type '.+' "
    r"\((?P<path>[^():]+\.dag):(?P<start>\d+)-(?P<end>\d+)\)$"
)
REPLACEMENTS = {"Some": "Present", "None": "Absent"}
FALLBACK_WINDOW_BYTES = 4096


@dataclass(frozen=True, order=True)
class Edit:
    path: Path
    start: int
    end: int
    old: str
    new: str


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


def parse_edits(output: str) -> list[Edit]:
    edits: set[Edit] = set()
    for line in output.splitlines():
        match = DIAG_RE.match(line.strip())
        if not match:
            continue
        variant = match.group("variant")
        rel_path = Path(match.group("path"))
        if rel_path.is_absolute() or ".." in rel_path.parts:
            raise RuntimeError(f"refusing suspicious diagnostic path: {rel_path}")
        edits.add(
            Edit(
                path=REPO_ROOT / rel_path,
                start=int(match.group("start")),
                end=int(match.group("end")),
                old=variant,
                new=REPLACEMENTS[variant],
            )
        )
    return sorted(edits, key=lambda e: (str(e.path), e.start, e.end, e.old))


def apply_edits(edits: list[Edit], dry_run: bool) -> int:
    by_path: dict[Path, list[Edit]] = {}
    for edit in edits:
        by_path.setdefault(edit.path, []).append(edit)

    applied = 0
    for path, path_edits in sorted(by_path.items()):
        text = path.read_bytes()
        replacements: list[Edit] = []
        for edit in sorted(path_edits, key=lambda e: e.start):
            old = edit.old.encode("utf-8")
            actual = text[edit.start : edit.end]
            if actual == old:
                replacements.append(edit)
                continue

            lo = max(0, edit.start - FALLBACK_WINDOW_BYTES)
            hi = min(len(text), edit.end + FALLBACK_WINDOW_BYTES)
            candidates: list[int] = []
            pos = text.find(old, lo, hi)
            while pos != -1:
                candidates.append(pos)
                pos = text.find(old, pos + 1, hi)

            if not candidates:
                raise RuntimeError(
                    f"{path.relative_to(REPO_ROOT)}:{edit.start}-{edit.end}: "
                    f"expected {edit.old!r}, found {actual.decode('utf-8', 'replace')!r}; "
                    f"no nearby {edit.old!r} token"
                )

            nearest_distance = min(abs(pos - edit.start) for pos in candidates)
            nearest = [pos for pos in candidates if abs(pos - edit.start) == nearest_distance]
            if len(nearest) != 1:
                raise RuntimeError(
                    f"{path.relative_to(REPO_ROOT)}:{edit.start}-{edit.end}: "
                    f"ambiguous nearby {edit.old!r} token candidates {nearest}"
                )
            replacements.append(
                Edit(
                    path=edit.path,
                    start=nearest[0],
                    end=nearest[0] + len(old),
                    old=edit.old,
                    new=edit.new,
                )
            )

        if not dry_run:
            next_text = text
            for edit in sorted(replacements, key=lambda e: e.start, reverse=True):
                next_text = (
                    next_text[: edit.start]
                    + edit.new.encode("utf-8")
                    + next_text[edit.end :]
                )
            path.write_bytes(next_text)
        applied += len(replacements)
    return applied


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Rewrite diagnosed legacy Optional Some/None match arms to Present/Absent."
    )
    parser.add_argument("--dry-run", action="store_true", help="report edits without writing files")
    args = parser.parse_args()

    output = run_regen_verify()
    edits = parse_edits(output)
    if not edits:
        print("normalize_optional_match_surface: no diagnosed Some/None Optional arms found")
        return 0

    applied = apply_edits(edits, dry_run=args.dry_run)
    mode = "would rewrite" if args.dry_run else "rewrote"
    print(f"normalize_optional_match_surface: {mode} {applied} diagnosed Optional arm token(s)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
