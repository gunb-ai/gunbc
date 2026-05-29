#!/usr/bin/env python3
"""Strip WHAT-comments from v2 .dag files (RELEASE_TODO.md §4, CODING.md).

Preserves: module identity (first header line), section === dividers and titles,
🟡/🟢/🔴 marks, Anchor:, and WHY-style comments (invariants, gates, dissolve, etc.).
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

DIVIDER_RE = re.compile(r"^\s*//\s*=+\s*$")

WHY_KEEP_RE = re.compile(
    r"(?:"
    r"[\U0001f7e1\U0001f7e2\U0001f534]"  # 🟡 🟢 🔴
    r"|Anchor:"
    r"|dissolve"
    r"|INVARIANT"
    r"|invariant"
    r"|Invariant\s+\d"
    r"|must not|must NOT|Must not"
    r"|Do NOT|do NOT|DO NOT"
    r"|workaround"
    r"|Gate:"
    r"|feature:"
    r"|fail closed|Fail closed"
    r"|non-obvious"
    r"|because\b"
    r"|otherwise look wrong"
    r"|compatibility"
    r"|tradeoff"
    r"|SG-\d+"
    r"|C-\d+:"
    r"|D\d+:"
    r"|Provisional"
    r"|predicate dissolution"
    r"|needs-more-work"
    r"|gated\b"
    r"|bounded substrate"
    r"|Early Detection"
    r"|Decidability"
    r"|audit\b"
    r")",
    re.IGNORECASE,
)

TODO_STRIP_RE = re.compile(r"^\s*//\s*TODO:", re.IGNORECASE)


def why_keep(text: str) -> bool:
    return bool(WHY_KEEP_RE.search(text))


def is_blank_comment(line: str) -> bool:
    s = line.strip()
    return s == "//" or s == ""


def split_code_and_comment(line: str) -> tuple[str, str | None]:
    in_str = False
    i = 0
    n = len(line)
    while i < n:
        c = line[i]
        if c == '"' and (i == 0 or line[i - 1] != "\\"):
            in_str = not in_str
            i += 1
            continue
        if not in_str and c == "/" and i + 1 < n and line[i + 1] == "/":
            return line[:i].rstrip(), line[i:]
        i += 1
    return line, None


def is_section_title_line(line: str) -> bool:
    if DIVIDER_RE.match(line) or is_blank_comment(line):
        return False
    m = re.match(r"^\s*//\s*(.+?)\s*$", line)
    return m is not None and m.group(1).strip() != ""


def process_file_lines(lines: list[str]) -> list[str]:
    out: list[str] = []
    past_header = False
    header_title_kept = False

    in_section = False
    section_title_kept = False
    seen_divider_in_section = False

    for line in lines:
        if not past_header:
            stripped = line.strip()
            if stripped and not stripped.startswith("//"):
                past_header = True

        code, comment = split_code_and_comment(line)

        if comment is not None and code.strip() == "":
            if is_blank_comment(line):
                continue

            if why_keep(line):
                out.append(line)
                if in_section:
                    section_title_kept = True
                continue

            if TODO_STRIP_RE.match(line):
                continue

            if DIVIDER_RE.match(line):
                if in_section and seen_divider_in_section:
                    # Closing divider.
                    out.append(line)
                    in_section = False
                    section_title_kept = False
                    seen_divider_in_section = False
                else:
                    # Opening divider (or first divider in a new section).
                    out.append(line)
                    in_section = True
                    section_title_kept = False
                    seen_divider_in_section = True
                continue

            if not past_header:
                if not header_title_kept and is_section_title_line(line):
                    out.append(line)
                    header_title_kept = True
                continue

            if in_section:
                if not section_title_kept and is_section_title_line(line):
                    out.append(line)
                    section_title_kept = True
                continue

            # Standalone WHAT comment.
            continue

        if comment is not None and code.strip() != "":
            if why_keep(comment):
                out.append(code + comment)
            elif code.strip():
                out.append(code)
            continue

        if line.strip():
            out.append(line)

    collapsed: list[str] = []
    blank_run = 0
    for line in out:
        if line.strip() == "":
            blank_run += 1
            if blank_run <= 2:
                collapsed.append(line)
        else:
            blank_run = 0
            collapsed.append(line)
    return collapsed


def transform_file(path: Path) -> tuple[int, int]:
    original = path.read_text(encoding="utf-8")
    lines = original.splitlines(keepends=True)
    if not lines:
        return 0, 0
    ends_with_nl = original.endswith("\n")
    body = [ln.rstrip("\n") + "\n" for ln in lines]
    new_body = process_file_lines(body)
    new_text = "".join(new_body)
    if ends_with_nl and not new_text.endswith("\n"):
        new_text += "\n"
    before = sum(1 for ln in lines if ln.strip().startswith("//"))
    after = sum(1 for ln in new_text.splitlines(keepends=True) if ln.strip().startswith("//"))
    if new_text != original:
        path.write_text(new_text, encoding="utf-8")
    return before, after


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    targets = sorted((root / "src" / "v2").glob("*.dag"))
    if not targets:
        print("no .dag files found", file=sys.stderr)
        return 1
    total_before = total_after = 0
    for path in targets:
        b, a = transform_file(path)
        total_before += b
        total_after += a
        print(f"{path.name}: {b} -> {a} comment lines")
    print(f"total: {total_before} -> {total_after}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
