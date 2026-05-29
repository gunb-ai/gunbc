#!/usr/bin/env python3
"""Strip WHAT-comments from v2 .dag files (RELEASE_TODO.md §4, CODING.md).

Preserves: module identity (first header line), section === dividers and titles,
🟡/🟢/🔴 marks, Anchor:, and WHY-style comments (invariants, gates, dissolve, etc.).
Multi-line WHY paragraphs are kept intact once any line in the paragraph matches WHY_KEEP_RE.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

DIVIDER_RE = re.compile(r"^\s*//\s*=+\s*$")
COMMENT_LINE_RE = re.compile(r"^\s*//")

WHY_KEEP_RE = re.compile(
    r"(?:"
    r"[\U0001f7e1\U0001f7e2\U0001f534]"  # 🟡 🟢 🔴
    r"|Anchor:"
    r"|dissolve"
    r"|INVARIANT"
    r"|invariant"
    r"|Invariant\s+\d"
    r"|must not|must NOT|Must not"
    r"|should not|should NOT|Should not"
    r"|Do NOT|do NOT|DO NOT"
    r"|workaround"
    r"|Gate:"
    r"|feature:"
    r"|fail closed|Fail closed"
    r"|fail-closes"
    r"|non-obvious"
    r"|otherwise look wrong"
    r"|compatibility"
    r"|tradeoff"
    r"|Soundness"
    r"|Note:"
    r"|Historical"
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
    r"|unsound"
    r"|ROADMAP"
    r"|single authority"
    r")",
    re.IGNORECASE,
)

TODO_STRIP_RE = re.compile(r"^\s*//\s*TODO:", re.IGNORECASE)


def why_keep(text: str) -> bool:
    return bool(WHY_KEEP_RE.search(text))


def is_blank_comment(line: str) -> bool:
    s = line.strip()
    return s == "//" or s == ""


def is_comment_only_line(line: str) -> bool:
    code, comment = split_code_and_comment(line)
    return comment is not None and code.strip() == ""


CONTINUATION_PREFIX_RE = re.compile(
    r"^(?:"
    r"[\)\],]"
    r"|field\)"
    r"|values "
    r"|ahead "
    r"|iteration "
    r"|documented "
    r"|would "
    r"|accept "
    r"|arity mismatches\)"
    r"|needed\.?"
    r"|of alias \("
    r"|the grounded "
    r"|is UNSOUND"
    r"|and must not pass"
    r"|used different parameter"
    r"|caller's param"
    r"|back to itself"
    r"|unsoundly promote"
    r"|making descent"
    r"|functions are O\(n\)"
    r"|do not leak"
    r")",
    re.IGNORECASE,
)

STANDALONE_KEEP_PREFIX_RE = re.compile(
    r"^\s*//\s*(?:"
    r"[\U0001f7e1\U0001f7e2\U0001f534]"
    r"|Gate:"
    r"|Note:"
    r"|dissolve-on:"
    r"|fail closed"
    r"|Fail closed"
    r")",
    re.IGNORECASE,
)


def is_orphan_comment_fragment(line: str, *, inline: bool = False) -> bool:
    """Drop mid-sentence debris (never applied inside an active WHY paragraph)."""
    m = re.match(r"^\s*//\s*(.*)$", line)
    if not m:
        return False
    if STANDALONE_KEEP_PREFIX_RE.match(line):
        return False
    text = m.group(1).strip()
    if not text:
        return False
    if CONTINUATION_PREFIX_RE.match(text):
        return True
    if text.endswith("-") or text.endswith(","):
        return True
    if re.match(r"^[\)\]]", text):
        return True
    if inline:
        return False
    if re.match(r"^[a-z(`]", text) and not text.startswith("http"):
        return True
    return False


def emit_line(text: str) -> str:
    return text.rstrip("\n") + "\n"


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
            return line[:i].rstrip(), line[i:].rstrip("\n")
        i += 1
    return line.rstrip("\n"), None


def is_section_title_line(line: str) -> bool:
    if DIVIDER_RE.match(line) or is_blank_comment(line):
        return False
    m = re.match(r"^\s*//\s*(.+?)\s*$", line)
    return m is not None and m.group(1).strip() != ""


def process_file_lines(lines: list[str]) -> list[str]:
    out: list[str] = []
    past_header = False
    first_header_line_kept = False
    comment_run: list[str] = []

    def flush_comment_run() -> None:
        nonlocal in_why_paragraph, in_feature_marker_block
        if not comment_run:
            return
        if any(why_keep(ln) for ln in comment_run):
            for ln in comment_run:
                out.append(emit_line(ln.rstrip("\n")))
            if any(
                re.search(r"[\U0001f7e1\U0001f7e2\U0001f534]|dissolve-on:", ln)
                for ln in comment_run
            ):
                in_feature_marker_block = True
        else:
            for ln in comment_run:
                _process_standalone_comment(ln)
        comment_run.clear()

    in_section = False
    section_title_kept = False
    seen_divider_in_section = False
    in_feature_marker_block = False
    in_why_paragraph = False

    def _process_standalone_comment(line: str) -> None:
        nonlocal in_why_paragraph, in_feature_marker_block, section_title_kept
        nonlocal in_section, seen_divider_in_section, first_header_line_kept, past_header

        if in_why_paragraph and COMMENT_LINE_RE.match(line):
            out.append(emit_line(line.rstrip("\n")))
            return

        if is_blank_comment(line):
            in_feature_marker_block = False
            in_why_paragraph = False
            return

        if in_feature_marker_block:
            out.append(emit_line(line.rstrip("\n")))
            return

        if not past_header:
            if not first_header_line_kept and COMMENT_LINE_RE.match(line):
                out.append(emit_line(line.rstrip("\n")))
                first_header_line_kept = True
                return
            if why_keep(line):
                out.append(emit_line(line.rstrip("\n")))
                return
            return

        if is_orphan_comment_fragment(line, inline=False):
            return

        if why_keep(line):
            out.append(emit_line(line.rstrip("\n")))
            in_why_paragraph = True
            if re.search(r"[\U0001f7e1\U0001f7e2\U0001f534]|dissolve-on:", line):
                in_feature_marker_block = True
            if in_section:
                section_title_kept = True
            return

        if TODO_STRIP_RE.match(line):
            return

        if DIVIDER_RE.match(line):
            in_why_paragraph = False
            if in_section and seen_divider_in_section:
                out.append(emit_line(line.rstrip("\n")))
                in_section = False
                section_title_kept = False
                seen_divider_in_section = False
            else:
                out.append(emit_line(line.rstrip("\n")))
                in_section = True
                section_title_kept = False
                seen_divider_in_section = True
            return

        if in_section:
            if not section_title_kept and is_section_title_line(line):
                out.append(emit_line(line.rstrip("\n")))
                section_title_kept = True
            return

    for line in lines:
        if not past_header:
            stripped = line.strip()
            if stripped and not stripped.startswith("//"):
                flush_comment_run()
                past_header = True

        code, comment = split_code_and_comment(line)

        if comment is not None and code.strip() == "":
            comment_run.append(line.rstrip("\n"))
            continue

        flush_comment_run()

        if comment is not None and code.strip() != "":
            in_why_paragraph = False
            full = code + " " + comment
            if why_keep(comment) and not is_orphan_comment_fragment(
                full, inline=True
            ):
                out.append(emit_line(full))
            elif code.strip():
                out.append(emit_line(code))
            continue

        if line.strip():
            in_feature_marker_block = False
            in_why_paragraph = False
            out.append(emit_line(line.rstrip("\n")))

    flush_comment_run()
    return out


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
