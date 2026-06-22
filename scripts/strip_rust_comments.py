#!/usr/bin/env python3
"""Rust-aware comment stripper for the codebase-wide comment ban (.rs seed).

Removes // line comments (incl. /// //! doc comments) and /* */ block comments
(NESTED, incl. /** /*!) while preserving string literals "...", raw strings
r"..." / r#"..."# (any hash count, incl. b-prefixed), byte strings b"...", char
literals 'a' / '\\n', and lifetimes/labels 'a / 'static / 'loop. Multi-line and
//-in-string content are preserved. Lines inside string/raw-string literals are
never reflowed.

TRANSITIONAL (DESIGN.md §6) — dissolve-on: the parser-wall lands (free comment
becomes a parse error). The Rust seed shrinks toward zero (DESIGN §7); when both
hold this stripper has no input and is deleted. Peer: scripts/strip_dag_comments.py.

Usage: scripts/strip_rust_comments.py FILE [FILE ...]
"""
import sys


def strip(src):
    out = []
    line_protected = [False]   # per output line: contains literal (string/raw/char) chars

    def emit(c):
        out.append(c)
        if c == "\n":
            line_protected.append(False)

    def protect():
        line_protected[-1] = True

    i, n = 0, len(src)
    while i < n:
        c = src[i]
        nxt = src[i + 1] if i + 1 < n else ""

        # raw string: optional b, 'r', N '#', '"'
        j = i
        if c in ("r", "b") and (c == "r" or nxt == "r" or nxt == '"'):
            k = i
            if src[k] == "b":
                k += 1
            if k < n and src[k] == "r":
                k += 1
                hashes = 0
                while k < n and src[k] == "#":
                    hashes += 1
                    k += 1
                if k < n and src[k] == '"':
                    # raw string body until '"' + hashes '#'
                    for p in range(i, k + 1):
                        emit(src[p])
                    protect()
                    k += 1
                    closer = '"' + "#" * hashes
                    while k < n:
                        if src[k] == "\n":
                            emit(src[k])
                            protect()
                            k += 1
                            continue
                        if src[k] == '"' and src[k:k + 1 + hashes] == closer:
                            for p in range(k, k + 1 + hashes):
                                emit(src[p])
                            k += 1 + hashes
                            break
                        emit(src[k])
                        protect()
                        k += 1
                    i = k
                    continue

        # byte string b"..."
        if c == "b" and nxt == '"':
            i += 1
            c = '"'
            emit("b")
            # fall through into string handling below

        if c == '"':
            emit('"')
            protect()
            i += 1
            esc = False
            while i < n:
                ch = src[i]
                emit(ch)
                if ch == "\n":
                    protect()
                if esc:
                    esc = False
                elif ch == "\\":
                    esc = True
                elif ch == '"':
                    i += 1
                    break
                i += 1
            continue

        if c == "'":
            # char literal vs lifetime/label
            # char: '\\<esc>...'  OR  '<one char>'
            if nxt == "\\":
                emit("'")
                protect()
                i += 1
                esc = False
                while i < n:
                    ch = src[i]
                    emit(ch)
                    if esc:
                        esc = False
                    elif ch == "\\":
                        esc = True
                    elif ch == "'":
                        i += 1
                        break
                    i += 1
                continue
            elif i + 2 < n and src[i + 2] == "'":
                # 'x'  -> char literal
                emit("'"); emit(nxt); emit("'")
                protect()
                i += 3
                continue
            else:
                # lifetime / label: 'a 'static 'loop  — emit the quote, stay normal
                emit("'")
                i += 1
                continue

        if c == "/" and nxt == "/":
            i += 2
            while i < n and src[i] != "\n":
                i += 1
            continue

        if c == "/" and nxt == "*":
            depth = 1
            i += 2
            while i < n and depth > 0:
                if src[i] == "/" and i + 1 < n and src[i + 1] == "*":
                    depth += 1
                    i += 2
                elif src[i] == "*" and i + 1 < n and src[i + 1] == "/":
                    depth -= 1
                    i += 2
                elif src[i] == "\n":
                    emit("\n")
                    i += 1
                else:
                    i += 1
            continue

        emit(c)
        i += 1

    lines = "".join(out).split("\n")
    if len(line_protected) < len(lines):
        line_protected += [False] * (len(lines) - len(line_protected))

    result = []
    prev_blank = False
    for idx, line in enumerate(lines):
        prot = line_protected[idx]
        line = line if prot else line.rstrip()
        is_blank = (line == "") and not prot
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
