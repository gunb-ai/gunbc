#!/usr/bin/env python3
"""String-aware .dag comment stripper.

Removes // line comments and /* */ block comments while preserving string
literals ("...") and char literals ('...'), including multi-line strings.
Drops lines that become empty due to comment removal and collapses runs of
blank lines, but never alters lines containing string-literal content.
"""
import sys

NORMAL, STRING, CHAR, LINE_C, BLOCK_C = range(5)


def strip(src: str) -> str:
    out = []
    # per output-line: does it contain string/char literal chars?
    line_has_str = [False]
    st = NORMAL
    esc = False
    i = 0
    n = len(src)

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
            # else: drop comment char
        elif st == BLOCK_C:
            if c == "\n":
                emit(c)  # preserve line structure
            elif c == "*" and nxt == "/":
                st = NORMAL
                i += 2
                continue
            # else: drop comment char
        i += 1

    text = "".join(out)
    lines = text.split("\n")
    # line_has_str has one entry per line (the trailing entry pairs the final line)
    if len(line_has_str) < len(lines):
        line_has_str += [False] * (len(lines) - len(line_has_str))

    cleaned = []
    for idx, line in enumerate(lines):
        protected = line_has_str[idx]
        if protected:
            cleaned.append(line)
            continue
        stripped = line.rstrip()
        cleaned.append(stripped)

    # collapse runs of blank lines (only among non-protected lines)
    result = []
    prev_blank = False
    for idx, line in enumerate(cleaned):
        protected = line_has_str[idx] if idx < len(line_has_str) else False
        is_blank = (line == "") and not protected
        if is_blank and prev_blank:
            continue
        result.append(line)
        prev_blank = is_blank

    # strip leading blank lines
    while result and result[0] == "":
        result.pop(0)
    # ensure single trailing newline
    while len(result) >= 2 and result[-1] == "" and result[-2] == "":
        result.pop()
    text = "\n".join(result)
    if not text.endswith("\n"):
        text += "\n"
    return text


if __name__ == "__main__":
    for path in sys.argv[1:]:
        with open(path, "r") as f:
            src = f.read()
        new = strip(src)
        with open(path, "w") as f:
            f.write(new)
