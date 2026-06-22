#!/usr/bin/env python3
"""String-literal-aware comment stripper for the codebase-wide comment ban.

Removes // line and /* */ block comments from .dag while preserving "..."
string and '...' char literals (including multi-line strings and the //-in-URL
class) and code structure. Shared by every fan-out subtree so the literal-aware
logic is single-authority, not re-implemented per child (divergence would
corrupt the //-in-string files).

TRANSITIONAL (DESIGN.md §6) — dissolve-on: the parser-wall lands (free // becomes
a parse error in 02_parse/syntax). Once comments are unwritable by construction,
this stripper has no input class and is deleted.

Usage: scripts/strip_dag_comments.py FILE [FILE ...]
"""
import sys

NORMAL, STRING, CHAR, LINE_C, BLOCK_C = range(5)


def strip(src):
    out = []
    line_has_str = [False]
    st = NORMAL
    esc = False
    i, n = 0, len(src)

    def emit(c):
        out.append(c)
        if c == "\n":
            line_has_str.append(False)

    while i < n:
        c = src[i]
        nxt = src[i + 1] if i + 1 < n else ""
        if st == NORMAL:
            if c == '"':
                st = STRING
                line_has_str[-1] = True
                emit(c)
            elif c == "'":
                st = CHAR
                line_has_str[-1] = True
                emit(c)
            elif c == "/" and nxt == "/":
                st = LINE_C
                i += 2
                continue
            elif c == "/" and nxt == "*":
                st = BLOCK_C
                i += 2
                continue
            else:
                emit(c)
        elif st == STRING:
            emit(c)
            if c == "\n":
                line_has_str[-1] = True
            if esc:
                esc = False
            elif c == "\\":
                esc = True
            elif c == '"':
                st = NORMAL
        elif st == CHAR:
            emit(c)
            if esc:
                esc = False
            elif c == "\\":
                esc = True
            elif c == "'":
                st = NORMAL
        elif st == LINE_C:
            if c == "\n":
                st = NORMAL
                emit(c)
        elif st == BLOCK_C:
            if c == "\n":
                emit(c)
            elif c == "*" and nxt == "/":
                st = NORMAL
                i += 2
                continue
        i += 1

    lines = "".join(out).split("\n")
    if len(line_has_str) < len(lines):
        line_has_str += [False] * (len(lines) - len(line_has_str))

    result = []
    prev_blank = False
    for idx, line in enumerate(lines):
        protected = line_has_str[idx]
        line = line if protected else line.rstrip()
        is_blank = (line == "") and not protected
        if is_blank and prev_blank:
            continue
        result.append(line)
        prev_blank = is_blank

    while result and result[0] == "":
        result.pop(0)
    while len(result) >= 2 and result[-1] == "" and result[-2] == "":
        result.pop()
    text = "\n".join(result)
    return text if text.endswith("\n") else text + "\n"


if __name__ == "__main__":
    for path in sys.argv[1:]:
        with open(path) as f:
            src = f.read()
        new = strip(src)
        if new != src:
            with open(path, "w") as f:
                f.write(new)
